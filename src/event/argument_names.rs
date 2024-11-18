/* HOW TO USE
    These are the keys used in args and msg_args fields of event responses.

    All values are used to tweak an entity's default behavior to make it distinct from other entity types.
    If left unset, default values will be used in event calculations. These are set by the individual event
    implementations.

    Response functions and event implementations are allowed to fill in default values to non-overrides if 
    no yaml-specified value is present in order to modify the behavior of other events or responses.

    Override values, and only override values, are used for temporary changes to an entity's behavior. Do not
    modify a non-override argument of an event response at run time unless the changes is permanent, and 
    non-reversible. 
    
    When modifying an override value, ensure that a duration effect is added that will clean up the change on removal. 
        (TODO: these are not fully implemented yet)
 */


// FLOAT ARGUMENTS
pub const ARG_DAMAGE_MULTIPLIER: &'static str = "DMG_MULTIPLIER";
pub const ARG_DAMAGE_MULTIPLIER_OVERRIDE: &'static str = "DMG_MULTIPLIER_OVERRIDE";

// MESSAGE ARGUMENTS
pub const MSG_ARG_ATTACKER: &'static str = "ATTACKER_ENTITY";
pub const MSG_ARG_ATTACK_MESSAGE: &'static str = "ATTACK_MSG";
pub const MSG_ARG_ADDENDUM: &'static str = "EVENT_ADDENDUM";
pub const MSG_ARG_ADDENDUM_OVERRIDE: &'static str = "EVENT_ADDENDUM_OVERRIDE";