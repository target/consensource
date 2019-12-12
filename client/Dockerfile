FROM node:8 as builder

COPY protos /protos
COPY client /client

WORKDIR /client

RUN cd /client \
    && npm install \
    && npm run build-dev

FROM httpd:2.4-alpine

COPY --from=builder /client /client

# Remove default index.html
RUN rm /usr/local/apache2/htdocs/index.html

# Copy client over
RUN cp -rf /client/public/* /usr/local/apache2/htdocs

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
    KeepAlive On\n\
    KeepAliveTimeout 600\n\
    ProxyTimeout 600\n\
    <Directory ~ '^/[\w+\d+-]+'>\n\
    FallbackResource /index.html\n\
    </Directory>\n\
    <Files ~ '\.(js|css|gif|jpe?g|png)$'>\n\
    FallbackResource disabled\n\
    ErrorDocument 404 'File not found'\n\
    </Files>\n\
    \n\
    " >>/usr/local/apache2/conf/httpd.conf

EXPOSE 80/tcp
