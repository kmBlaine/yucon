#!/usr/bin/bash

echo $'Installing Yucon...'
echo 'Checking current user...'

if whoami|grep -v root; then
	echo $'\nThis installer requires root privelege to run.\nFailed to install. Exiting';
	exit;
fi

echo 'Root privelege detected. Proceeding...'
echo 'Creating config file path...'

mkdir --parents /etc/yucon/

echo 'Copying units.dat file...'

cp ./cfg/units.dat /etc/yucon/

echo 'Copying binary...'

cp ./bin/yucon-linux /usr/bin/
mv /usr/bin/yucon-linux /usr/bin/yucon
chown root:users /usr/bin/yucon
chmod 755 /usr/bin/yucon

echo $'\nYucon installed successfully!'
