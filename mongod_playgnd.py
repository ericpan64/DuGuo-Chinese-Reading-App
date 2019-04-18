# Get MongoDB to work in Python
# Tutorial Link: http://api.mongodb.com/python/3.6.0/tutorial.html
from pymongo import MongoClient
# import datetime as dt
client = MongoClient("mongodb://localhost:27017/") # Default IP address
db = client.test_database
collection = db.test_colletion

test_list = [{"user": "Eric", "body": "Ni hao 你好"},
            {"user": "Billy", "body": "为蛇么", "newParam": 1}]

docs = collection.insert_many(test_list) # Bulk insert
for doc in collection.find():  # Bulk query
    print(doc)

# Testing ID query code
# test_id = collection.insert_one(test).inserted_id
# print(test_id)
# print(collection.find_one({"_id":test_id}))