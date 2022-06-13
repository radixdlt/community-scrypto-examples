Red='\033[1;31m'          # Red
Green='\033[1;32m'        # Green
Yellow='\033[0;33m'       # Yellow
Blue='\033[1;34m'         # Blue
Purple='\033[0;35m'       # Purple
Cyan='\033[1;36m'         # Cyan
NC='\033[0m'              # No Color

logr () {
    >&2 echo -e "$Red === $@ ===$NC"
}

logg () {
    >&2 echo -e "$Green === $@ ===$NC"
}

logc () {
    >&2 echo -e "$Cyan === $@ ===$NC"
}

logp () {
    >&2 echo -e "$Purple == $@ ==$NC"
}

logy () {
    >&2 echo -e "$Yellow === $@ ===$NC"
}


completed () {
    >&2 echo -e "$Green========================$NC"
    >&2 echo -e "$Green====== COMPELETED ======$NC"
    >&2 echo -e "$Green========================$NC"
}