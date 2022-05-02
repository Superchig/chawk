#!/bin/awk -f

$2 == "F" {
  sum += ($1 - 32) / 1.8
}

$2 == "C" {
  sum += $1
}

END {
  print sum / (NR - 1) " C"
}
