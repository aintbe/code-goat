# You need root privileges to run this script
# Set "privileged: true" in docker-compose.yaml.
sudo dmesg --since "-10m" -T -L | grep 'sig' | grep 'syscall'