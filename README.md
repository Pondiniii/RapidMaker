jak użyć purgecss:
1. pobierz node.js i npm sudo apt install to gowno
2. npm install purgecss --save-dev
https://github.com/FullHuman/purgecss
3. /home/pc/node_modules/.bin/purgecss --config purgecss.config.js


Docker tutorial:
sudo docker build -t rapidmaker .
sudo docker run rapidmaker


Zapisz i przenieś docker file:
docker save -o rapidmaker.tar rapidmaker

odczyt:
docker load -i rapidmaker.tar