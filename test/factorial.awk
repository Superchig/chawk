function factorial(n) {
  if (n <= 0) {
    return 1
  } else {
    return n * factorial(n - 1)
  }
}

END {
  print "Factorial of 8: " factorial(8)
}
