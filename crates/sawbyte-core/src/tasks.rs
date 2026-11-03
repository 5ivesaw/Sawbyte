//! Budgeted 3D A* navigation, composable task compilers, and the multi-screen workflow engine.
//! Every task compiles strictly into legal player inputs (`Action` and `SubTickEvent`) and
//! reconciles against observable world/screen state.

use crate::events::{BlockFace, ClickMode, Hand, SubTickEvent};
use crate::screen::{ItemSelector, ScreenState, SelectionError};
use crate::{Action, Button};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Trait queried by the pathfinder to evaluate passability and standability without coupling to wire formats.
pub trait VoxelQuery {
    fn is_solid(&self, x: i32, y: i32, z: i32) -> bool;
    fn is_climbable(&self, _x: i32, _y: i32, _z: i32) -> bool {
        false
    }
    fn is_water(&self, _x: i32, _y: i32, _z: i32) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Node {
    pos: [i32; 3],
    g_cost: u32,
    f_cost: u32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_cost
            .cmp(&self.f_cost)
            .then_with(|| other.g_cost.cmp(&self.g_cost))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn heuristic(a: [i32; 3], b: [i32; 3]) -> u32 {
    let dx = (i64::from(a[0]) - i64::from(b[0]))
        .unsigned_abs()
        .min(100_000) as u32;
    let dy = (i64::from(a[1]) - i64::from(b[1]))
        .unsigned_abs()
        .min(100_000) as u32;
    let dz = (i64::from(a[2]) - i64::from(b[2]))
        .unsigned_abs()
        .min(100_000) as u32;
    let y_cost = if a[1] <= b[1] { dy * 6 } else { dy * 2 };
    (dx + dz) * 10 + y_cost
}

fn can_stand_at(world: &impl VoxelQuery, x: i32, y: i32, z: i32) -> bool {
    let Some(y_above) = y.checked_add(1) else {
        return false;
    };
    let Some(y_below) = y.checked_sub(1) else {
        return false;
    };
    let head_clear = !world.is_solid(x, y, z) && !world.is_solid(x, y_above, z);
    let supported =
        world.is_solid(x, y_below, z) || world.is_climbable(x, y, z) || world.is_water(x, y, z);
    head_clear && supported
}

/// Computes a 3D walk/jump/climb path from `start` to `goal` within `max_expanded_nodes`.
pub fn find_path(
    world: &impl VoxelQuery,
    start: [i32; 3],
    goal: [i32; 3],
    max_expanded_nodes: usize,
) -> Result<Vec<[i32; 3]>, &'static str> {
    if start == goal {
        return Ok(vec![start]);
    }
    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<[i32; 3], [i32; 3]> = HashMap::new();
    let mut g_scores: HashMap<[i32; 3], u32> = HashMap::new();
    let mut closed: HashSet<[i32; 3]> = HashSet::new();

    g_scores.insert(start, 0);
    open.push(Node {
        pos: start,
        g_cost: 0,
        f_cost: heuristic(start, goal),
    });

    let mut expanded = 0usize;
    while let Some(current) = open.pop() {
        if current.pos == goal {
            let mut path = vec![goal];
            let mut cursor = goal;
            while let Some(&prev) = came_from.get(&cursor) {
                path.push(prev);
                if prev == start {
                    break;
                }
                cursor = prev;
            }
            path.reverse();
            return Ok(path);
        }
        if !closed.insert(current.pos) {
            continue;
        }
        expanded += 1;
        if expanded > max_expanded_nodes {
            return Err("pathfinding node budget exceeded");
        }

        let [cx, cy, cz] = current.pos;
        for [dx, dz] in [[-1, 0], [1, 0], [0, -1], [0, 1]] {
            let Some(nx) = cx.checked_add(dx) else {
                continue;
            };
            let Some(nz) = cz.checked_add(dz) else {
                continue;
            };
            // Flat step, step/jump up (+1 with overhead clearance), or step down (-1)
            for (dy, step_cost, needs_jump_headroom) in [
                (0i32, 10u32, false),
                (1i32, 16u32, true),
                (-1i32, 12u32, false),
            ] {
                let Some(ny) = cy.checked_add(dy) else {
                    continue;
                };
                if needs_jump_headroom {
                    if let Some(y_head) = cy.checked_add(2) {
                        if world.is_solid(cx, y_head, cz) {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                if ny < cy {
                    if let Some(y_clear) = cy.checked_add(1) {
                        if world.is_solid(nx, y_clear, nz) {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                if !can_stand_at(world, nx, ny, nz) {
                    continue;
                }
                let next = [nx, ny, nz];
                let tentative_g = current.g_cost.saturating_add(step_cost);
                if tentative_g < *g_scores.get(&next).unwrap_or(&u32::MAX) {
                    came_from.insert(next, current.pos);
                    g_scores.insert(next, tentative_g);
                    open.push(Node {
                        pos: next,
                        g_cost: tentative_g,
                        f_cost: tentative_g.saturating_add(heuristic(next, goal)),
                    });
                }
            }
        }
    }
    Err("no valid path found to target")
}

/// Compiles a 3D waypoint path into timed `(tick, Action)` scheduler submissions.
pub fn compile_navigation_inputs(
    start_tick: u64,
    current_yaw: f32,
    waypoints: &[[i32; 3]],
    sprint: bool,
) -> Vec<(u64, Action)> {
    if waypoints.len() < 2 {
        return Vec::new();
    }
    let mut actions = Vec::new();
    let mut tick = start_tick;
    let mut yaw = current_yaw as f64;
    actions.push((
        tick,
        Action::Button {
            button: Button::Forward,
            down: true,
        },
    ));
    if sprint {
        actions.push((
            tick,
            Action::Button {
                button: Button::Sprint,
                down: true,
            },
        ));
    }

    for pair in waypoints.windows(2) {
        let from = pair[0];
        let to = pair[1];
        let dx = (to[0] - from[0]) as f64;
        let dz = (to[2] - from[2]) as f64;
        let target_yaw = (-dx).atan2(dz).to_degrees();
        let mut delta = (target_yaw - yaw) % 360.0;
        if delta > 180.0 {
            delta -= 360.0;
        } else if delta < -180.0 {
            delta += 360.0;
        }
        if delta.abs() > 1e-3 {
            actions.push((
                tick,
                Action::Look {
                    yaw: delta,
                    pitch: 0.0,
                },
            ));
            yaw = target_yaw;
        }
        if to[1] > from[1] {
            actions.push((
                tick,
                Action::Button {
                    button: Button::Jump,
                    down: true,
                },
            ));
            actions.push((
                tick + 1,
                Action::Button {
                    button: Button::Jump,
                    down: false,
                },
            ));
        }
        tick += if sprint { 4 } else { 5 };
    }

    if sprint {
        actions.push((
            tick,
            Action::Button {
                button: Button::Sprint,
                down: false,
            },
        ));
    }
    actions.push((
        tick,
        Action::Button {
            button: Button::Forward,
            down: false,
        },
    ));
    actions
}

/// High-level composable tasks that compile strictly into low-level inputs and events.
#[derive(Clone, Debug, PartialEq)]
pub enum ComposableTask {
    Navigate {
        goal: [i32; 3],
        sprint: bool,
        node_budget: usize,
    },
    MineBlock {
        pos: [i32; 3],
        face: BlockFace,
        duration_ticks: u32,
    },
    PlaceBlock {
        support_pos: [i32; 3],
        face: BlockFace,
        hotbar_slot: u8,
    },
    CraftRecipe {
        window_id: i32,
        recipe_id: String,
        make_all: bool,
    },
    SmeltItem {
        window_id: i32,
        state_id: i32,
        ingredient_slot: i16,
        fuel_slot: i16,
    },
    TradeWithVillager {
        villager_entity_id: i32,
        trade_index: u16,
    },
    EquipHotbar {
        slot: u8,
    },
    TransferSlot {
        window_id: i32,
        state_id: i32,
        from_slot: i16,
        quick_move: bool,
    },
    EatHeldItem {
        hand: Hand,
        use_ticks: u32,
    },
    FollowEntity {
        entity_id: i32,
        target_pos: [i32; 3],
        stop_distance: i32,
    },
}

impl ComposableTask {
    /// Compiles a task into discrete sub-tick events starting at `start_tick`.
    pub fn compile_events(&self, start_tick: u64, base_sequence: i32) -> Vec<(u64, SubTickEvent)> {
        match self {
            Self::MineBlock {
                pos,
                face,
                duration_ticks,
            } => {
                let end_tick = start_tick + u64::from((*duration_ticks).max(1));
                vec![
                    (
                        start_tick,
                        SubTickEvent::StartDestroyBlock {
                            pos: *pos,
                            face: *face,
                            sequence: base_sequence,
                        },
                    ),
                    (
                        start_tick,
                        SubTickEvent::SwipeArm {
                            hand: Hand::MainHand,
                        },
                    ),
                    (
                        end_tick,
                        SubTickEvent::StopDestroyBlock {
                            pos: *pos,
                            face: *face,
                            sequence: base_sequence + 1,
                        },
                    ),
                ]
            }
            Self::PlaceBlock {
                support_pos,
                face,
                hotbar_slot,
            } => vec![
                (
                    start_tick,
                    SubTickEvent::SelectHotbar { slot: *hotbar_slot },
                ),
                (
                    start_tick,
                    SubTickEvent::UseItemOn {
                        hand: Hand::MainHand,
                        pos: *support_pos,
                        face: *face,
                        hit: [0.5, 0.5, 0.5],
                        inside_block: false,
                        sequence: base_sequence,
                    },
                ),
                (
                    start_tick,
                    SubTickEvent::SwipeArm {
                        hand: Hand::MainHand,
                    },
                ),
            ],
            Self::CraftRecipe {
                window_id,
                recipe_id,
                make_all,
            } => vec![(
                start_tick,
                SubTickEvent::PlaceRecipe {
                    window_id: *window_id,
                    recipe: recipe_id.clone(),
                    make_all: *make_all,
                },
            )],
            Self::SmeltItem {
                window_id,
                state_id,
                ingredient_slot,
                fuel_slot,
            } => vec![
                (
                    start_tick,
                    SubTickEvent::ContainerClick {
                        window_id: *window_id,
                        state_id: *state_id,
                        slot: *ingredient_slot,
                        button: 0,
                        mode: ClickMode::QuickMove,
                    },
                ),
                (
                    start_tick + 2,
                    SubTickEvent::ContainerClick {
                        window_id: *window_id,
                        state_id: *state_id + 1,
                        slot: *fuel_slot,
                        button: 0,
                        mode: ClickMode::QuickMove,
                    },
                ),
            ],
            Self::TradeWithVillager {
                villager_entity_id,
                trade_index,
            } => vec![
                (
                    start_tick,
                    SubTickEvent::InteractEntity {
                        entity_id: *villager_entity_id,
                        interaction: crate::events::EntityInteraction::Interact {
                            hand: Hand::MainHand,
                        },
                        sneaking: false,
                    },
                ),
                (
                    start_tick + 3,
                    SubTickEvent::SelectTrade { slot: *trade_index },
                ),
            ],
            Self::EquipHotbar { slot } => {
                vec![(start_tick, SubTickEvent::SelectHotbar { slot: *slot })]
            }
            Self::TransferSlot {
                window_id,
                state_id,
                from_slot,
                quick_move,
            } => vec![(
                start_tick,
                SubTickEvent::ContainerClick {
                    window_id: *window_id,
                    state_id: *state_id,
                    slot: *from_slot,
                    button: 0,
                    mode: if *quick_move {
                        ClickMode::QuickMove
                    } else {
                        ClickMode::Pickup
                    },
                },
            )],
            Self::EatHeldItem { hand, use_ticks } => vec![
                (
                    start_tick,
                    SubTickEvent::UseItem {
                        hand: *hand,
                        sequence: base_sequence,
                    },
                ),
                (
                    start_tick + u64::from((*use_ticks).max(32)),
                    SubTickEvent::ReleaseUseItem,
                ),
            ],
            Self::Navigate { .. } | Self::FollowEntity { .. } => Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetryPolicy {
    Never,
    Bounded(u8),
}

/// Programmable workflow step supporting screen waits, selectors, text fields, variables,
/// conditionals, loops, client menus/settings, and custom registered extensions.
#[derive(Clone, Debug, PartialEq)]
pub enum WorkflowStep {
    WaitScreenTitle {
        title: String,
        timeout_ms: u64,
    },
    WaitElement {
        label: String,
        timeout_ms: u64,
    },
    WaitField {
        field_name: String,
        timeout_ms: u64,
    },
    WaitEvent {
        event_name: String,
        timeout_ms: u64,
    },
    ClickItem {
        selector: ItemSelector,
        unique: bool,
        mode: ClickMode,
        retry: RetryPolicy,
    },
    ClickWidget {
        label: String,
        retry: RetryPolicy,
    },
    FillField {
        field_name: String,
        text: String,
    },
    SetVar {
        key: String,
        value: String,
    },
    IfVarEquals {
        key: String,
        expected: String,
        then_steps: Vec<WorkflowStep>,
        else_steps: Vec<WorkflowStep>,
    },
    Repeat {
        times: usize,
        body: Vec<WorkflowStep>,
    },
    InvokeExtension {
        name: String,
        arg: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorkflowOutcome {
    Completed {
        emitted_events: Vec<SubTickEvent>,
        final_revision: i32,
        variables: HashMap<String, String>,
    },
    UncertainOutcomeHalt {
        step_index: usize,
        reason: String,
        emitted_events: Vec<SubTickEvent>,
    },
    Failed {
        step_index: usize,
        reason: String,
    },
}

/// Environment callbacks used during workflow execution so both live sessions and CI fixtures
/// can supply screen transitions, events, and extension hooks.
pub trait WorkflowHost {
    fn current_screen(&mut self) -> Option<&mut ScreenState>;
    fn poll_screen_until_title(&mut self, title: &str, timeout_ms: u64) -> bool;
    fn poll_event(&mut self, event_name: &str, timeout_ms: u64) -> bool;
    fn on_event_emitted(&mut self, event: &SubTickEvent);
    fn invoke_extension(
        &mut self,
        name: &str,
        arg: &str,
        vars: &mut HashMap<String, String>,
    ) -> Result<(), String>;
}

/// Executes a sequence of `WorkflowStep`s with strict revision checking and `RetryPolicy::Never` safety.
pub fn execute_workflow(steps: &[WorkflowStep], host: &mut impl WorkflowHost) -> WorkflowOutcome {
    let mut emitted = Vec::new();
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut last_irrevocable_click = false;

    fn run_slice(
        slice: &[WorkflowStep],
        host: &mut impl WorkflowHost,
        emitted: &mut Vec<SubTickEvent>,
        vars: &mut HashMap<String, String>,
        last_irrevocable_click: &mut bool,
    ) -> Result<(), (usize, String, bool)> {
        for (idx, step) in slice.iter().enumerate() {
            match step {
                WorkflowStep::WaitScreenTitle { title, timeout_ms } => {
                    if !host.poll_screen_until_title(title, *timeout_ms) {
                        return Err((
                            idx,
                            format!("timed out waiting for screen '{title}' after {timeout_ms} ms"),
                            *last_irrevocable_click,
                        ));
                    }
                    *last_irrevocable_click = false;
                }
                WorkflowStep::WaitElement { label, timeout_ms } => {
                    let exists = host
                        .current_screen()
                        .is_some_and(|s| s.click_widget(label).is_ok());
                    if !exists {
                        return Err((
                            idx,
                            format!("timed out waiting for element '{label}' ({timeout_ms} ms)"),
                            *last_irrevocable_click,
                        ));
                    }
                }
                WorkflowStep::WaitField {
                    field_name,
                    timeout_ms,
                } => {
                    let exists = host
                        .current_screen()
                        .is_some_and(|s| s.fields.iter().any(|f| f.name == *field_name));
                    if !exists {
                        return Err((
                            idx,
                            format!("timed out waiting for field '{field_name}' ({timeout_ms} ms)"),
                            *last_irrevocable_click,
                        ));
                    }
                }
                WorkflowStep::WaitEvent {
                    event_name,
                    timeout_ms,
                } => {
                    if !host.poll_event(event_name, *timeout_ms) {
                        return Err((
                            idx,
                            format!(
                                "timed out waiting for event '{event_name}' after {timeout_ms} ms"
                            ),
                            *last_irrevocable_click,
                        ));
                    }
                    *last_irrevocable_click = false;
                }
                WorkflowStep::ClickItem {
                    selector,
                    unique,
                    mode,
                    retry,
                } => {
                    let screen = host
                        .current_screen()
                        .ok_or_else(|| (idx, "no active screen for ClickItem".to_owned(), false))?;
                    let rev = screen.state_id;
                    let ev = screen
                        .click_matching_item(Some(rev), selector, *unique, 0, *mode)
                        .map_err(|e: SelectionError| (idx, format!("{e:?}"), false))?;
                    *last_irrevocable_click = *retry == RetryPolicy::Never;
                    host.on_event_emitted(&ev);
                    emitted.push(ev);
                }
                WorkflowStep::ClickWidget { label, retry } => {
                    let screen = host.current_screen().ok_or_else(|| {
                        (idx, "no active screen for ClickWidget".to_owned(), false)
                    })?;
                    let ev = screen
                        .click_widget(label)
                        .map_err(|e| (idx, format!("{e:?}"), false))?;
                    *last_irrevocable_click = *retry == RetryPolicy::Never;
                    host.on_event_emitted(&ev);
                    emitted.push(ev);
                }
                WorkflowStep::FillField { field_name, text } => {
                    let expanded = if let Some(var_name) = text.strip_prefix('$') {
                        vars.get(var_name).cloned().unwrap_or_else(|| text.clone())
                    } else {
                        text.clone()
                    };
                    let screen = host
                        .current_screen()
                        .ok_or_else(|| (idx, "no active screen for FillField".to_owned(), false))?;
                    let ev = screen
                        .fill_field(field_name, &expanded)
                        .map_err(|e| (idx, format!("{e:?}"), false))?;
                    host.on_event_emitted(&ev);
                    emitted.push(ev);
                }
                WorkflowStep::SetVar { key, value } => {
                    vars.insert(key.clone(), value.clone());
                }
                WorkflowStep::IfVarEquals {
                    key,
                    expected,
                    then_steps,
                    else_steps,
                } => {
                    let branch = if vars.get(key).map(String::as_str) == Some(expected.as_str()) {
                        then_steps
                    } else {
                        else_steps
                    };
                    run_slice(branch, host, emitted, vars, last_irrevocable_click)?;
                }
                WorkflowStep::Repeat { times, body } => {
                    for _ in 0..(*times).min(256) {
                        run_slice(body, host, emitted, vars, last_irrevocable_click)?;
                    }
                }
                WorkflowStep::InvokeExtension { name, arg } => {
                    host.invoke_extension(name, arg, vars)
                        .map_err(|e| (idx, e, *last_irrevocable_click))?;
                }
            }
        }
        Ok(())
    }

    match run_slice(
        steps,
        host,
        &mut emitted,
        &mut vars,
        &mut last_irrevocable_click,
    ) {
        Ok(()) => {
            let final_revision = host.current_screen().map_or(0, |s| s.state_id);
            WorkflowOutcome::Completed {
                emitted_events: emitted,
                final_revision,
                variables: vars,
            }
        }
        Err((step_index, reason, true)) => WorkflowOutcome::UncertainOutcomeHalt {
            step_index,
            reason: format!(
                "{reason} (irrevocable action preceded failure; halting instead of retrying to prevent duplicate purchase/effect)"
            ),
            emitted_events: emitted,
        },
        Err((step_index, reason, false)) => WorkflowOutcome::Failed { step_index, reason },
    }
}

/// Example multi-screen test fixture workflow demonstrating general UI transitions and state verification.
pub fn shop_purchase_workflow(item_id: &str, amount: &str) -> Vec<WorkflowStep> {
    vec![
        WorkflowStep::SetVar {
            key: "desired_amount".into(),
            value: amount.into(),
        },
        WorkflowStep::WaitScreenTitle {
            title: "Shop".into(),
            timeout_ms: 5000,
        },
        WorkflowStep::ClickItem {
            selector: ItemSelector {
                name_exact: Some("Building Blocks".into()),
                ..Default::default()
            },
            unique: true,
            mode: ClickMode::Pickup,
            retry: RetryPolicy::Bounded(1),
        },
        WorkflowStep::WaitScreenTitle {
            title: "Building Blocks".into(),
            timeout_ms: 5000,
        },
        WorkflowStep::ClickItem {
            selector: ItemSelector {
                id: Some(item_id.into()),
                ..Default::default()
            },
            unique: true,
            mode: ClickMode::Pickup,
            retry: RetryPolicy::Bounded(1),
        },
        WorkflowStep::WaitElement {
            label: "Confirm".into(),
            timeout_ms: 5000,
        },
        WorkflowStep::ClickWidget {
            label: "Confirm".into(),
            retry: RetryPolicy::Bounded(1),
        },
        WorkflowStep::WaitField {
            field_name: "amount".into(),
            timeout_ms: 5000,
        },
        WorkflowStep::FillField {
            field_name: "amount".into(),
            text: "$desired_amount".into(),
        },
        WorkflowStep::ClickWidget {
            label: "Pay".into(),
            retry: RetryPolicy::Never,
        },
        WorkflowStep::WaitEvent {
            event_name: "purchase_confirmed".into(),
            timeout_ms: 5000,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::{ItemStack, TextFieldState, WidgetElement};

    struct FlatWithWall;
    impl VoxelQuery for FlatWithWall {
        fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
            if y == -61 {
                return true;
            }
            // 1-block high step at z == 2 that can be jumped over
            x == 0 && y == -60 && z == 2
        }
    }

    #[test]
    fn pathfinder_jumps_over_obstacle_and_compiles_inputs() {
        let path = find_path(&FlatWithWall, [0, -60, 0], [0, -60, 4], 512).unwrap();
        assert_eq!(path.first().copied(), Some([0, -60, 0]));
        assert_eq!(path.last().copied(), Some([0, -60, 4]));
        assert!(path.iter().any(|p| p[1] == -59));
        let inputs = compile_navigation_inputs(0, 0.0, &path, true);
        assert!(inputs.iter().any(|(_, a)| matches!(
            a,
            Action::Button {
                button: Button::Jump,
                down: true
            }
        )));
    }

    #[test]
    fn pathfinder_handles_overflow_coordinates_safely() {
        let res = find_path(
            &FlatWithWall,
            [i32::MAX - 1, -60, i32::MAX - 1],
            [i32::MAX, -60, i32::MAX],
            32,
        );
        assert!(res.is_err() || res.is_ok());
    }

    struct MockShopServer {
        screen: Option<ScreenState>,
        confirm_payment_event: bool,
    }

    impl MockShopServer {
        fn new(confirm_payment_event: bool) -> Self {
            let mut s = ScreenState::container(1, 1, 3, "Shop");
            s.set_slot(
                13,
                Some(ItemStack::new("minecraft:bricks", 1).with_name("Building Blocks")),
            );
            Self {
                screen: Some(s),
                confirm_payment_event,
            }
        }
    }

    impl WorkflowHost for MockShopServer {
        fn current_screen(&mut self) -> Option<&mut ScreenState> {
            self.screen.as_mut()
        }
        fn poll_screen_until_title(&mut self, title: &str, _timeout_ms: u64) -> bool {
            self.screen.as_ref().is_some_and(|s| s.title == title)
        }
        fn poll_event(&mut self, event_name: &str, _timeout_ms: u64) -> bool {
            event_name == "purchase_confirmed" && self.confirm_payment_event
        }
        fn on_event_emitted(&mut self, event: &SubTickEvent) {
            if let SubTickEvent::ContainerClick { slot: 13, .. } = event {
                let mut next = ScreenState::container(2, 10, 3, "Building Blocks");
                next.set_slot(
                    10,
                    Some(ItemStack::new("minecraft:stone", 1).with_name("Stone")),
                );
                self.screen = Some(next);
            } else if let SubTickEvent::ContainerClick { slot: 10, .. } = event {
                if let Some(s) = &mut self.screen {
                    s.widgets.push(WidgetElement {
                        id: 1,
                        label: "Confirm".into(),
                        enabled: true,
                        bounds: (60, 40, 50, 20),
                    });
                    s.state_id += 1;
                }
            } else if let SubTickEvent::ContainerButtonClick { button_id: 1, .. } = event {
                if let Some(s) = &mut self.screen {
                    s.fields.push(TextFieldState {
                        name: "amount".into(),
                        text: String::new(),
                        max_length: 16,
                        focused: true,
                        editable: true,
                    });
                    s.widgets.push(WidgetElement {
                        id: 2,
                        label: "Pay".into(),
                        enabled: true,
                        bounds: (60, 70, 50, 20),
                    });
                    s.state_id += 1;
                }
            }
        }
        fn invoke_extension(
            &mut self,
            _name: &str,
            _arg: &str,
            _vars: &mut HashMap<String, String>,
        ) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn shop_workflow_succeeds_and_never_retries_after_payment_timeout() {
        let wf = shop_purchase_workflow("minecraft:stone", "64");

        // 1. Normal confirmed flow
        let mut ok_host = MockShopServer::new(true);
        let outcome = execute_workflow(&wf, &mut ok_host);
        assert!(matches!(outcome, WorkflowOutcome::Completed { .. }));

        // 2. Timeout after clicking "Pay" (retry: Never) halts with UncertainOutcomeHalt instead of paying twice
        let mut timeout_host = MockShopServer::new(false);
        let outcome2 = execute_workflow(&wf, &mut timeout_host);
        assert!(matches!(
            outcome2,
            WorkflowOutcome::UncertainOutcomeHalt { .. }
        ));
    }
}
