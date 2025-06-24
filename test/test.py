import requests
import time

base = "http://localhost:8000"

def login():
    c = requests.post(base+"/signin", json={"username": "cx@email.com", 
                                    "password": "Password1!"}).json()
    return c["data"]["token"]


def create_new_crash(t):
    def acc_data():
        return {"instant": 1234, "x": 60.1, "y": 90.2, "z": 9.81}

    def gps_data():
        return {
            "instant": 1633024800,
            "latitude": 51.5074,
            "longitude": -0.1278,
            "altitude": 35.0,
            "speed": 12.5
        }
    d = {
        "timestamp": 1750648151700,  # int(time.time()*1000),
        "accel_data": [acc_data() for _ in range(200)],
        "gps_data": [gps_data() for _ in range(5)]
    }
    print(requests.post(base+"/crashes/new", json=d,
                    headers={"Authorization": "Bearer "+t}).text)

def get_crashes(t):
    r = requests.get(base+"/crashes/all", headers={"Authorization": "Bearer "+t})
    print(r.text)
    return r.json()

tok = login()
create_new_crash(tok)
get_crashes(tok)
#create_new_crash(tok)