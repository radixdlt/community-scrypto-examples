# Define helper functions

function Reload {
  resim publish --address $PACKAGE .
}

function Write-Output {
  echo "$1"
}

function Get-Account-Address {
  grep 'Account address' | cut -d ":" -f2 | xargs
}

function Get-Public-Key {
  grep 'Public key' | cut -d: -f 2 | xargs
}

function Get-Component {
  grep 'Component' | cut -d: -f 2 | xargs
}

function Get-Resource-Def {
  grep "name: \"$1\"" | sed -r 's/.*resource_def: (\w+).*/\1/'
}

function Get-Resource-Amount {
  grep "name: \"$1\"" | sed -r 's/.*amount: ([0-9]+).*/\1/'
}

function Get-New-Def {
  grep 'Resource' | cut -d: -f 2 | xargs
}

function Get-Market-Price {
  grep "$1" | cut -d'|' -f3 | xargs
}

function Wait-For-User {
  echo
  read -n 1 -s -r -p "$1"
  echo
}

function Exit-Unless-Equal {
  if [ "$1" != "$2" ]; then
    echo
    echo "$3" 1>&2
    return 1
  fi
}

# Lastly, convert variable assignments in ps1 script to bash.
# e.g.
#
#   $FOO = echo hallo
#
# into
#
#   export FOO=`echo hallo`
#
# The rest is already valid bash due to the use of the helper functions above.
#
# We use export (and global variables in ps1) so users can inspect the saved values
# after sourcing the demo script.
BASH_STEPS_FILE=`tempfile`
cat ./tests/steps.ps1 | sed -e 's/^\$/export /g' -e 's/ = /=`/g' -e '/=`/s/.$/`/g' | tr -d "\r" > $BASH_STEPS_FILE

set -e

source $BASH_STEPS_FILE

# @TODO Figure out a way to do this WITHOUT exiting the bash shell if the steps fail.
# Although this is really only a problem if the demo fails which it shouldn't!
