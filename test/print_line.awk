# This pattern-block pair matches on all lines and does nothing
// { }
# This pattern-block pair will use the default action, which is printing
//
# This pattern-block pair will use the default pattern, which is nothing
{ }

# This will print the same output for each line
{ print "This line will be repeated!" }

{
  print "This line will also be repeated."
  print "And this line will follow."
}

{
  print $1
}

/word/ {
  print $1
}
