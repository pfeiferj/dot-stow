SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR
cd ..

unameOut="$(uname -s)"
case "${unameOut}" in
    Linux*)     machine=Linux;;
    Darwin*)    machine=Mac;;
    CYGWIN*)    machine=Cygwin;;
    MINGW*)     machine=MinGw;;
    *)          machine="UNKNOWN:${unameOut}"
esac
echo ${machine}


if [ "$machine" == "Linux" ]; then
  rm -f linux-target.zip
  curl -s https://api.github.com/repos/pfeiferj/dot-stow/releases/latest \
  | grep "linux-target.zip" \
  | cut -d : -f 2,3 \
  | tr -d \" \
  | wget -qi -
  unzip linux-target.zip
  chmod +x dot-stow
fi

if [ "$machine" == "Mac" ]; then
  rm -f mac-target.zip
  curl -s https://api.github.com/repos/pfeiferj/dot-stow/releases/latest \
  | grep "mac-target.zip" \
  | cut -d : -f 2,3 \
  | tr -d \" \
  | wget -qi -
  unzip mac-target.zip
  chmod +x dot-stow
fi
