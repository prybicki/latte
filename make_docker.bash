
# Katalog z kodem źródłowym + testy
MOUNT_HOST_DIR=$(pwd)

# Zbuduj obraz z ./Dockerfile
sudo docker build -t latte-prybicki .

# Uruchom obraz dockera
sudo docker run -it --mount type=bind,source=${MOUNT_HOST_DIR},target=/latte latte-prybicki:latest

# Wewnątrz dockera:
# cd /latte
# make
