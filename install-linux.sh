#!/bin/bash

echo $'Installing Yucon...\n'
echo 'Checking current user...'

if whoami|grep -v root; then
	echo $'\nThis installer requires root privelege to run.\nFailed to install. Exiting.';
	exit;
fi

echo 'Root privelege detected. Proceeding...'
echo 'Creating config file path: /etc/yucon/'

mkdir --parents /etc/yucon/

echo 'Copying units.cfg file to config path...'

cp ./units.cfg /etc/yucon/

echo 'Copying binary...'

cp ./yucon-linux /usr/bin/
mv /usr/bin/yucon-linux /usr/bin/yucon
chown root:root /usr/bin/yucon
chmod 755 /usr/bin/yucon

echo $'\nDone'
