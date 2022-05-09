function factorial(n) {
  if (n <= 0) {
    return 1
  } else {
    local prev = factorial(n - 1)
    print "in function - prev: " prev
    return n * prev
  }
}

END {
  prev = -1

  print "before function - prev: " prev

  print "Factorial of 8: " factorial(8)

  print "after function - prev: " prev
}
