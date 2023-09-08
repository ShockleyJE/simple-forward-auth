#!/bin/bash
set -x
set -eo pipefail

# Get current working directory
cwd=$(pwd)

# Check if current directory is /usr/src/mybuild
if [ "$cwd" != "/usr/src/mybuild" ]; then
    # If not, change directory to /usr/src/mybuild/authn_service
    cd /usr/src/mybuild
    echo >&2 "Moved to: /usr/src/mybuild"
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install --version='~0.6' sqlx-cli --no-default-features --features rustls,postgres"
  echo >&2 "to install it."
  exit 1
fi

>&2 echo "Applying migration!"
#sqlx database create
sqlx migrate run

>&2 echo "Applied migrations! - running application now"
echo >&2 "Moved to: $cwd"
cd $cwd

./target/release/authn_service