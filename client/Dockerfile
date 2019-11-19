FROM httpd:2.4-alpine

RUN echo -e "\
    \n\
    ServerName consensource\n\
    AddDefaultCharset utf-8\n\
    LoadModule proxy_module modules/mod_proxy.so\n\
    LoadModule proxy_http_module modules/mod_proxy_http.so\n\
    ProxyPass /api/block-stream http://rest-api:9010/push/0\n\
    ProxyPassReverse /api/block-stream http://rest-api:9010/push/0\n\
    ProxyPass /api http://rest-api:9009/api connectiontimeout=300 timeout=300\n\
    ProxyPassReverse /api http://rest-api:9009/api\n\
    \n\
    " >>/usr/local/apache2/conf/httpd.conf

EXPOSE 80/tcp
