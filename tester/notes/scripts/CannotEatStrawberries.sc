require (CanEat)

listen ConsumeFoodEvent {
    if (event.food is Strawberry) {
        event.cancel()
    }
}