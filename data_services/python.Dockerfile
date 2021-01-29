# Find a way to slim this down later...
FROM python:3.8.7
RUN pip install spacy pandas pypinyin pymongo dnspython opencc redis
RUN python -m spacy download zh_core_web_sm
WORKDIR /token-server
COPY . .
RUN chmod +x startup.sh
EXPOSE 8881
# Note: make sure EOL sequence is LF (Windows default is CRLF)
CMD ["./startup.sh"]