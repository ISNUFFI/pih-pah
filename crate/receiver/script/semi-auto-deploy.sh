# Check for help flag
default_db_link="postgres://postgres:postgres@localhost:5433/pih-pah"

bin="receiver"
service="pih-pah-${bin}"
dir="$(dirname "$(realpath "$0")")/"
remote_dir="/home/${USER}/pih-pah-deploy/${bin}/"

cd "${dir}../" || exit 1
. ../../script/log.sh

if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
  # echo "Can start only from project folder"
  printf "Usage: %s [-h]" "${dir}"
  printf "Environment Variables:"
  printf "  SSH_USER\tSet the SSH destination as user\n"
  printf "  SSH_SERVER\tSet the SSH destination as server address\n"
  printf "  SSH_USER_PASSWORD\tpassword for user in remote host, I hope you do not use root\n"
  printf "  SSH_PRIVATE_KEY\tSsh private key\n"
  printf "  DATABASE_URL\tpostgresql link\n"
  printf ""
  printf "  defaults:"
  printf "  \tDATABASE_URL: %s" "${default_db_link}"
  exit 0
fi

log "${dir} running..."
log "check env..."

if [ -z "${SSH_USER}" ]; then
  error 'USER must be set. Exiting.'
  exit 1
fi

if [ -z "${SSH_SERVER}" ]; then
  error 'SSH_SERVER must be set. Exiting.'
  exit 1
fi

if [ -z "${SSH_PRIVATE_KEY}" ]; then
  error 'SSH_PRIVATE_KEY must be set. Exiting.'
  exit 1
fi

if [ -z "${SSH_USER_PASSWORD}" ]; then
  error 'SSH_USER_PASSWORD must be set. Exiting.'
  exit 1
fi

if [ -z "${DATABASE_URL}" ]; then
  DATABASE_URL="${default_db_link}"
fi

# Use an environment variable for the SSH user and server
SSH_DEST="${SSH_USER}@${SSH_SERVER}"

log 'building...'
cargo build --release
env CARGO_TARGET_DIR=../../target cargo build --release --bin server

# Ssh setup
tmp_ssh_private="$(mktemp)"
echo "${SSH_PRIVATE_KEY}" > "${tmp_ssh_private}"

# Transfer the Rust binary
log 'some ssh magic...'
ssh -i "${tmp_ssh_private}" "${SSH_DEST}" "mkdir -p ${remote_dir} && rm -f ${remote_dir}/${bin}" # if not exist
scp -i "${tmp_ssh_private}" "${dir}../../../target/release/${bin}" "${SSH_DEST}:${remote_dir}"


# SSH and setup service
log 'connecting to server...'

temp_service="$(mktemp)"

ssh -i "${tmp_ssh_private}" "${SSH_DEST}" << "EOF"
  chmod +x  ${remote_dir}${bin}

  echo "[Unit]
Description=pih-pah ${bin}

[Service]
ExecStart=env DATABASE_URL=${DATABASE_URL} ${remote_dir}/${bin} 2007
Restart=always

[Install]
WantedBy=multi-user.target" > ${temp_file}

  printf '%s' "${SSH_USER_PASSWORD}" | sudo -S -rm -f /etc/systemd/system/${service}.service
  printf '%s' "${SSH_USER_PASSWORD}" | sudo -S mv ${temp_service} /etc/systemd/system/${service}.service
  printf '%s' "${SSH_USER_PASSWORD}" | sudo -S systemctl daemon-reload
  printf '%s' "${SSH_USER_PASSWORD}" | sudo -S systemctl enable ${service}
  printf '%s' "${SSH_USER_PASSWORD}" | sudo -S systemctl start ${service}
  printf '%s' "${SSH_USER_PASSWORD}" | sudo -S systemctl restart ${service}
EOF

rm -f "${temp_service}"
rm -f "${tmp_ssh_private}"