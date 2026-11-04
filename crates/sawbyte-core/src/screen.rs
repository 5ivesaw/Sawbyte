//! Complete screen, container, widget, text-field, and item-stack state machine with
//! revision-checked (`state_id`) selectors and click/fill compilers.

use crate::events::{ClickMode, SubTickEvent};

/// Structured representation of a Minecraft 26.3 item stack and its data components.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemStack {
    pub id: String,
    pub count: u32,
    pub display_name: Option<String>,
    pub lore: Vec<String>,
    pub enchantments: Vec<(String, i32)>,
    pub damage: Option<(i32, i32)>, // (current_damage, max_damage)
    pub custom_model_data: Option<i32>,
    pub components: Vec<(String, String)>,
    pub components_raw: Vec<u8>,
}

impl ItemStack {
    pub fn new(id: impl Into<String>, count: u32) -> Self {
        Self {
            id: id.into(),
            count,
            display_name: None,
            lore: Vec::new(),
            enchantments: Vec::new(),
            damage: None,
            custom_model_data: None,
            components: Vec::new(),
            components_raw: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn with_lore(mut self, lines: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.lore = lines.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_component(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.components.push((key.into(), value.into()));
        self
    }
}

/// Every supported vanilla container, editor, dialog, out-of-game client menu, and custom mod screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScreenKind {
    PlayerInventory,
    GenericContainer { rows: u8 },
    CraftingTable,
    Furnace,
    BlastFurnace,
    Smoker,
    Anvil,
    EnchantingTable,
    BrewingStand,
    VillagerTrading,
    Stonecutter,
    Loom,
    CartographyTable,
    Grindstone,
    SmithingTable,
    Beacon,
    Hopper,
    DropperOrDispenser,
    ShulkerBox,
    Crafter3x3,
    Lectern,
    SignEditor { front: bool, pos: [i32; 3] },
    BookEditor { signing: bool },
    ServerDialog { dialog_id: String },
    ClientMainMenu,
    ClientSingleplayerMenu,
    ClientMultiplayerMenu,
    ClientOptionsMenu,
    ClientPauseMenu,
    CustomModScreen { screen_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WidgetElement {
    pub id: i32,
    pub label: String,
    pub enabled: bool,
    pub bounds: (i32, i32, i32, i32), // (x, y, width, height)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextFieldState {
    pub name: String,
    pub text: String,
    pub max_length: usize,
    pub focused: bool,
    pub editable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TradeOffer {
    pub index: u16,
    pub base_cost_a: ItemStack,
    pub cost_b: Option<ItemStack>,
    pub result: ItemStack,
    pub uses: i32,
    pub max_uses: i32,
    pub out_of_stock: bool,
}

/// Revision-tracked snapshot of the currently active screen or container.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenState {
    pub window_id: i32,
    pub state_id: i32,
    pub kind: ScreenKind,
    pub title: String,
    pub slots: Vec<Option<ItemStack>>,
    pub cursor_item: Option<ItemStack>,
    pub widgets: Vec<WidgetElement>,
    pub fields: Vec<TextFieldState>,
    pub trades: Vec<TradeOffer>,
    pub properties: Vec<(i16, i16)>,
}

impl ScreenState {
    pub fn container(window_id: i32, state_id: i32, rows: u8, title: impl Into<String>) -> Self {
        let total_slots = (rows as usize * 9) + 36;
        Self {
            window_id,
            state_id,
            kind: ScreenKind::GenericContainer { rows },
            title: title.into(),
            slots: vec![None; total_slots],
            cursor_item: None,
            widgets: Vec::new(),
            fields: Vec::new(),
            trades: Vec::new(),
            properties: Vec::new(),
        }
    }

    pub fn set_slot(&mut self, slot: usize, item: Option<ItemStack>) {
        if slot < self.slots.len() {
            self.slots[slot] = item;
            self.state_id = self.state_id.wrapping_add(1).max(1);
        }
    }

    /// Updates a slot from an authoritative server packet without locally incrementing `state_id`.
    pub fn set_slot_authoritative(&mut self, slot: usize, item: Option<ItemStack>) {
        if slot >= self.slots.len() {
            self.slots.resize(slot + 1, None);
        }
        self.slots[slot] = item;
    }

    /// Atomically replaces the screen's slot table, carried cursor item, and authoritative server `state_id`.
    pub fn apply_server_content(
        &mut self,
        state_id: i32,
        slots: Vec<Option<ItemStack>>,
        cursor_item: Option<ItemStack>,
    ) {
        self.state_id = state_id;
        self.slots = slots;
        self.cursor_item = cursor_item;
    }
}

/// Composable item/slot selector for GUI automation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemSelector {
    pub id: Option<String>,
    pub name_exact: Option<String>,
    pub name_contains: Option<String>,
    pub lore_contains: Option<String>,
    pub component_match: Option<(String, String)>,
    pub min_count: Option<u32>,
    pub slot_range: Option<(usize, usize)>,
}

impl ItemSelector {
    pub fn matches(&self, slot_index: usize, item: &ItemStack) -> bool {
        if let Some((start, end)) = self.slot_range {
            if slot_index < start || slot_index > end {
                return false;
            }
        }
        if let Some(id) = &self.id {
            if &item.id != id {
                return false;
            }
        }
        if let Some(exact) = &self.name_exact {
            if item.display_name.as_deref() != Some(exact.as_str()) {
                return false;
            }
        }
        if let Some(sub) = &self.name_contains {
            if !item
                .display_name
                .as_deref()
                .is_some_and(|n| n.contains(sub.as_str()))
            {
                return false;
            }
        }
        if let Some(lore_sub) = &self.lore_contains {
            if !item.lore.iter().any(|l| l.contains(lore_sub.as_str())) {
                return false;
            }
        }
        if let Some((k, v)) = &self.component_match {
            if !item.components.iter().any(|(ck, cv)| ck == k && cv == v) {
                return false;
            }
        }
        if let Some(min) = self.min_count {
            if item.count < min {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionError {
    NoScreenOpen,
    RevisionMismatch { expected: i32, actual: i32 },
    NotFound(String),
    AmbiguousMatch { count: usize, slots: Vec<usize> },
    WidgetDisabled(String),
    FieldNotEditable(String),
}

impl ScreenState {
    /// Finds all slot indices matching `selector`, or enforces uniqueness when `unique == true`.
    pub fn select_slot(
        &self,
        selector: &ItemSelector,
        unique: bool,
        occurrence: Option<usize>,
    ) -> Result<usize, SelectionError> {
        let matches: Vec<usize> = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(idx, slot)| {
                slot.as_ref()
                    .filter(|item| selector.matches(idx, item))
                    .map(|_| idx)
            })
            .collect();

        if matches.is_empty() {
            return Err(SelectionError::NotFound(format!(
                "no item matched selector {selector:?} on screen '{}'",
                self.title
            )));
        }
        if unique && matches.len() > 1 {
            return Err(SelectionError::AmbiguousMatch {
                count: matches.len(),
                slots: matches,
            });
        }
        let pick = occurrence.unwrap_or(0);
        matches.get(pick).copied().ok_or_else(|| {
            SelectionError::NotFound(format!(
                "occurrence {pick} out of bounds ({})",
                matches.len()
            ))
        })
    }

    /// Compiles a revision-checked `ContainerClick` event for a matched slot.
    pub fn click_matching_item(
        &self,
        expected_revision: Option<i32>,
        selector: &ItemSelector,
        unique: bool,
        button: i8,
        mode: ClickMode,
    ) -> Result<SubTickEvent, SelectionError> {
        if let Some(expected) = expected_revision {
            if self.state_id != expected {
                return Err(SelectionError::RevisionMismatch {
                    expected,
                    actual: self.state_id,
                });
            }
        }
        let slot = self.select_slot(selector, unique, None)?;
        Ok(SubTickEvent::ContainerClick {
            window_id: self.window_id,
            state_id: self.state_id,
            slot: slot as i16,
            button,
            mode,
        })
    }

    /// Compiles a button click on a named widget element (or slot-backed menu button).
    pub fn click_widget(&self, label: &str) -> Result<SubTickEvent, SelectionError> {
        let matched: Vec<&WidgetElement> =
            self.widgets.iter().filter(|w| w.label == label).collect();
        if matched.is_empty() {
            // Also check if a container item acts as a button with that display name
            let sel = ItemSelector {
                name_exact: Some(label.to_owned()),
                ..Default::default()
            };
            if let Ok(slot) = self.select_slot(&sel, true, None) {
                return Ok(SubTickEvent::ContainerClick {
                    window_id: self.window_id,
                    state_id: self.state_id,
                    slot: slot as i16,
                    button: 0,
                    mode: ClickMode::Pickup,
                });
            }
            return Err(SelectionError::NotFound(format!(
                "widget '{label}' not found on screen '{}'",
                self.title
            )));
        }
        if matched.len() > 1 {
            return Err(SelectionError::AmbiguousMatch {
                count: matched.len(),
                slots: matched.iter().map(|w| w.id as usize).collect(),
            });
        }
        let widget = matched[0];
        if !widget.enabled {
            return Err(SelectionError::WidgetDisabled(label.to_owned()));
        }
        Ok(SubTickEvent::ContainerButtonClick {
            window_id: self.window_id,
            button_id: widget.id,
        })
    }

    /// Updates a named text field (e.g. Anvil rename, custom dialog amount prompt, or Sign/Dialog input) and emits the appropriate event.
    pub fn fill_field(
        &mut self,
        field_name: &str,
        text: &str,
    ) -> Result<SubTickEvent, SelectionError> {
        let field = self
            .fields
            .iter_mut()
            .find(|f| f.name == field_name)
            .ok_or_else(|| {
                SelectionError::NotFound(format!(
                    "field '{field_name}' not found on screen '{}'",
                    self.title
                ))
            })?;
        if !field.editable || text.chars().count() > field.max_length {
            return Err(SelectionError::FieldNotEditable(field_name.to_owned()));
        }
        field.text = text.to_owned();
        field.focused = true;
        Ok(SubTickEvent::RenameItem {
            name: text.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_rejects_ambiguous_items_and_enforces_revision() {
        let mut screen = ScreenState::container(1, 10, 3, "Building Blocks");
        screen.set_slot(
            11,
            Some(
                ItemStack::new("minecraft:stone", 64)
                    .with_name("Stone Pack")
                    .with_lore(["Price: $100"]),
            ),
        );
        screen.set_slot(
            15,
            Some(
                ItemStack::new("minecraft:stone", 64)
                    .with_name("Stone Pack")
                    .with_lore(["Price: $250"]),
            ),
        );

        let ambiguous = ItemSelector {
            id: Some("minecraft:stone".into()),
            ..Default::default()
        };
        assert_eq!(
            screen.select_slot(&ambiguous, true, None),
            Err(SelectionError::AmbiguousMatch {
                count: 2,
                slots: vec![11, 15]
            })
        );

        // Disambiguating by lore succeeds and compiles a revision-safe click
        let cheap_stone = ItemSelector {
            id: Some("minecraft:stone".into()),
            lore_contains: Some("Price: $100".into()),
            ..Default::default()
        };
        let rev = screen.state_id;
        let click = screen
            .click_matching_item(Some(rev), &cheap_stone, true, 0, ClickMode::Pickup)
            .unwrap();
        assert_eq!(
            click,
            SubTickEvent::ContainerClick {
                window_id: 1,
                state_id: rev,
                slot: 11,
                button: 0,
                mode: ClickMode::Pickup,
            }
        );

        // Stale revision fails explicitly
        assert_eq!(
            screen.click_matching_item(Some(rev - 1), &cheap_stone, true, 0, ClickMode::Pickup),
            Err(SelectionError::RevisionMismatch {
                expected: rev - 1,
                actual: rev
            })
        );
    }
}
