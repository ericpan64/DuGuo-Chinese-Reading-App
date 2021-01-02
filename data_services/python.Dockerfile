# Find a way to slim this down later...
FROM python:3.8.7
RUN pip install spacy pandas pypinyin pymongo
RUN python -m spacy download zh_core_web_sm
# Copying relative to directory with docker-compose.yml
COPY ./data_services .
RUN chmod +x /startup.sh
CMD ["/startup.sh"]