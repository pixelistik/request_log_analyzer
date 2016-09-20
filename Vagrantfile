Vagrant.configure(2) do |config|
  config.vm.box = "ubuntu/xenial64"
  config.vm.provider "virtualbox" do |v|
    v.memory = 2048
    v.cpus = 2
  end

  $script = <<SCRIPT
if [ ! -e /home/vagrant/.cargo/bin/rustup ]
then
    # DNS needs fixing...
    sudo rm /etc/resolv.conf
    sudo ln -s /run/resolvconf/resolv.conf /etc/resolv.conf
    sudo service resolvconf reload

    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable

    sudo apt-get install build-essential
    rustup target add x86_64-unknown-linux-musl
fi
SCRIPT

config.vm.provision "shell", inline: $script, privileged: false
end
