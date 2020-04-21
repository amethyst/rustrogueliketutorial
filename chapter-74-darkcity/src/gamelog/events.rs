use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref EVENTS : Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
}

pub fn clear_events() {
    EVENTS.lock().unwrap().clear();
}

pub fn record_event<T: ToString>(event: T, n : i32) {
    let event_name = event.to_string();
    let mut events_lock = EVENTS.lock();
    let events = events_lock.as_mut().unwrap();
    if let Some(e) = events.get_mut(&event_name) {
        *e += n;
    } else {
        events.insert(event_name, n);
    }
}

pub fn get_event_count<T: ToString>(event: T) -> i32 {
    let event_name = event.to_string();
    let events_lock = EVENTS.lock();
    let events = events_lock.unwrap();
    if let Some(e) = events.get(&event_name) {
        *e
    } else {
        0
    }
}

pub fn clone_events() -> HashMap<String, i32> {
    EVENTS.lock().unwrap().clone()
}

pub fn load_events(events : HashMap<String, i32>) {
    EVENTS.lock().unwrap().clear();
    events.iter().for_each(|(k,v)| {
        EVENTS.lock().unwrap().insert(k.to_string(), *v);
    });
}