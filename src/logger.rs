use std::cell::RefCell;
use std::collections::VecDeque;

use crate::component::Name;

thread_local!(
    pub static LOG: MessageLog = MessageLog::new();
);

/* ATTACK LOG FORMATS
generate_attack_message:
    [attacker_name] [hit_message] [defender_name] [damage listing] ([addendum])

    Example:
        [Bat] [bit] [Hero] [for 3 damage.] [You are now poisoned.]

generate_take_damage_message:
    [defender_name] [damage listing] ([addendum])

    Example:
        [Bat] [took 15 damage.] [It's super effective.]

Currently addendums are only used to indicate damage resistance or vulnerabilities.
If a None value is passed for a name, it will be substituted with "Someone".

The templates refering to burning or fire are deprecated.
 */ 


pub struct MessageLog {
    message_queue: RefCell<VecDeque<String>>,
}

impl MessageLog {
    pub fn new() -> Self {
        MessageLog {
            message_queue: RefCell::new(VecDeque::new()),
        }
    }

    pub fn queue_message(&self, msg: &str) {
        self.message_queue.borrow_mut().push_back(msg.to_string());
    }

    pub fn next_message(&self) -> Option<String> {
        self.message_queue.borrow_mut().pop_front()
    }
}

pub fn log_message(msg: &str) {
    LOG.with(|log| log.queue_message(msg));
}

pub fn generate_attack_message(
    attacker: Option<&Name>,
    defender: Option<&Name>,
    hit_message: &str,
    addendum: &str,
    damage_taken: u32,
) -> String {
    let attacker_name = match attacker {
        Some(Name(name)) => name.as_str(),
        None => "Someone",
    };

    let defender_name = match defender {
        Some(Name(name)) => name.as_str(),
        None => "Someone",
    };

    let damage_message = format!("for {damage_taken} damage."); 

    vec![
        attacker_name,
        hit_message,
        defender_name,
        &damage_message,
        addendum,
    ]
    .join(" ")
}

pub fn generate_take_damage_message(
    defender: Option<&Name>, 
    damage_taken: u32,
    addendum: &str,
) -> String {
    let defender_name = match defender {
        Some(Name(name)) => name.as_str(),
        None => "Someone",
    };
    vec![defender_name, "took", &damage_taken.to_string(), "damage.", addendum].join(" ")
}

pub fn generate_receive_gold_message(amount: u32) -> String {
    vec!["You found", &amount.to_string(), "gold!"].join(" ")
}

pub fn generate_is_burning_message(defender: Option<&Name>, damage_taken: u32) -> String {
    let defender_name = match defender {
        Some(Name(name)) => name.as_str(),
        None => "Someone",
    };
    vec![
        defender_name,
        "is burning! Took",
        &damage_taken.to_string(),
        "damage.",
    ]
    .join(" ")
}

pub fn generate_on_fire_message(defender: Option<&Name>) -> String {
    let defender_name = match defender {
        Some(Name(name)) => name.as_str(),
        None => "Someone",
    };
    vec![defender_name, "catches on fire!"].join(" ")
}

pub fn generate_wake_up_message(name: Option<&Name>) -> String {
    let name = match name {
        Some(Name(name)) => name,
        None => "Someone",
    };

    vec![&name, "woke up!"].join(" ")
}

pub fn generate_sleep_message(name: Option<&Name>) -> String {
    let name = match name {
        Some(Name(name)) => name,
        None => "Someone",
    };
    
    vec![&name, "doesn't notice you."].join(" ")
}
