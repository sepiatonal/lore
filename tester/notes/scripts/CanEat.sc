data {
    hunger: Decimal = 1.0,
}

event ConsumeFood(food: Food) {
    self.hunger = self.hunger + ev.item.food_value
    ev.item.removeOne()
}

on ItemInteract {
    if (ev.item is Food) {
        ConsumeFood!(food: ev.item)
    }
}