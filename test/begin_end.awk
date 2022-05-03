BEGIN { print "This is at the beginning." }

END { print "This is at the end." }

{ print $1 }
