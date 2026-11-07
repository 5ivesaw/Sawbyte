use sawbyte_play::{inventory::PacketTable, world};
fn main() {
    let profile = world::default_26_3_profile();
    let mut all = serde_json::Map::new();
    for p in [766, 767, 768, 769, 770, 771, 772, 773, 774, 775, 776, 777] {
        let t = PacketTable::for_protocol(p, &profile);
        let dbg = format!("{:?}", t);
        all.insert(p.to_string(), serde_json::Value::String(dbg));
    }
    println!("{}", serde_json::Value::Object(all));
}
