require (CanEat)

listen ConsumeFoodEvent {
    if (event.food is RawMeat) {
        self.vomit()
    }
}

fun vomit() {
    // yknow
}