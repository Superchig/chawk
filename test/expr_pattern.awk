# Should print
2 == 2 { print $1 }

# Should not print
"" { print $1 }

# Should print
1 { print $1 }
