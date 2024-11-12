use crate::{
    component::responses::SpellResponse, error::Result,
};

use super::{Event, EventArguments};

pub struct SpellEvent {
    pub source: usize,
    pub effect: Box<dyn Fn(EventArguments) -> Result<()>>,
}

impl SpellEvent {
    pub fn new<F>(source: usize, effect: F) -> Self
    where F: Fn(EventArguments) -> Result<()> + 'static
    {
        Self { source, effect: Box::new(effect) }
    }
}

impl Event for SpellEvent {
    type Response = SpellResponse;
    fn apply(
        &self,
        event_data: EventArguments,
    ) -> Result<()> 
    {
        (self.effect) (event_data)
    }

    fn source(&self) -> usize {
        self.source
    }
}