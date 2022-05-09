END {
  x = "This is the initial value of x."

  1 && (x = "x has been modified.")

  print x

  y = "This is the initial value of y."

  0 && (y = "y has been modified.")

  print y
}
