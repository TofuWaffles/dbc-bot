use poise::serenity_prelude::json::{json,Value};

pub fn match_json() -> Value {
    let id1: i64 = 607102310526484480;
    let id2: i64 = 389504173714046976;
    let json: Value = json!([
      {
        "name": "Darkness",
        "tag": "RR82U9J0",
        "region": "APAC",
        "id": id1
      },
      {
        "name": "SpiderMat",
        "tag": "8QLUQ9292",
        "region": "EU",
        "id": id2
      }
    ]);

    json
}
