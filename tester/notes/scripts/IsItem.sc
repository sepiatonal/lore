data {
    stack_size: Number,
}

fun removeOne() {
    self.removeN(1)
}

fun removeN(n: Number) {
    if (self.stack_size == n) {
        do Destroy(self)
    } else if (self.stack_size < n) {
        warn("Attempted to remove more of a stack than exists!")
        do Destroy(self)
    } else {
        self.stack_size -= n
    }
}