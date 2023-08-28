use poise::serenity_prelude::json::{json, Value};

pub fn match_json() -> Value {
    let json: Value = json!([
      {
        "name": "Darkness",
        "tag": "#RR82U9J0",
        "region": "APAC",
        "id": "607102310526484480"
      },
      {
        "name": "🖤★|Ａ-Ｚ|★🥀",
        "tag": "#R0P2QR0Y",
        "region": "APAC",
        "id": "461143643298856960"
      }
    ]);

    json
}
