$2 ~ /F/ {
  print $1 " in Fahrenheit."
}

$2 ~ "F" {
  print $1 " in Fahrenheit, using a string like regex."
}

$2 !~ /C|unit/ {
  print $1 " in Fahrenheit, via !~ operator."
}

END {
  if ("foo 10" ~ 10) {
    print "First match at end."
  }

  if ("foo" ~ 10) {
    print "Second match at end."
  }
}
