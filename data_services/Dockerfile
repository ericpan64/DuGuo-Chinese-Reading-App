FROM python:3.8.12-slim-buster
WORKDIR /token-server
COPY requirements.txt .
RUN apt-get update &&\
    apt-get install -y gcc g++ &&\
    pip install -r requirements.txt &&\
    python -m spacy download zh_core_web_sm &&\
    apt-get remove -y gcc g++ &&\
    apt-get autoremove -y
COPY . .
RUN chmod +x startup.sh
EXPOSE 8881
# Note: make sure EOL sequence is LF (Windows default is CRLF)
CMD ["./startup.sh"]
