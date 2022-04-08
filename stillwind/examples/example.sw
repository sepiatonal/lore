event consume_food(food: Item, consumer: EatsFood) {
    destroy_one! (food)
    consumer.hunger = consumer.hunger + consumer.hunger_value
}
// TODO make the compiler/VM handle cascading events in the same tick, before mutation phase.
// this can be done by simulating the mutation (will need to implement crazy VM stuff) and
// checking which events get thrown.

// TODO an event effect should never cascade another effect in a nondeterministic way. for example:
// `if (consumer.x > 100) { consumer.y += 5 }` would be nondeterministic if a different event modifies x.

behavior EatsFood {
    // variables declared here persist upon any object that this behavior is attached to
    let hunger = 1.0

    // TODO parse where statements of this form (where field == self.EntityID)
    // so that instead of every individual entity checking, the dispatcher can
    // just not ever send the event to anyone but the specific entities. OR 
    // perhaps a `mentioning [EntityID]` modifier of the same sort of effect,
    // eg `interact_item? mentioning (self) where (interacter == self) {}`
    interact_item? where (interacter == self) {
        if (item is Food) {
            consume_food! (item, self)
        }
    }
}

behavior HateFatPeople {
    consume_food? {
        if (consumer.weight > 100) {
            print! ("Watch the BMI there, piggy!")
            cancel // cancel prevents the listened event from executing
        }
    }
}
// TODO global behaviors