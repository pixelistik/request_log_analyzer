Vagrant.configure(2) do |config|
  config.vm.box = "ubuntu/trusty64"
  $script = <<SCRIPT

if [ ! -e /home/vagrant/.cargo/bin/rustup ]
then
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
fi
SCRIPT

config.vm.provision "shell", inline: $script, privileged: false
end
