from flask import Flask
from flask import request
import requests
from bs4 import BeautifulSoup as bs


def parse(r, id: str) -> str:
    text = r.text
    soup = bs(text, "html.parser")
    line = soup.find(id=id)
    print(line)
    return line.get("value")


def login(username: str, password: str) -> int:
    # Get the initial login page with the sync token
    r_one = s.get('https://aggietime.usu.edu/login/auth')
    print(r_one.status_code)
    if r_one.status_code == 200:
        print('Succesful initial!')
    else:
        print("failure at initial")
    sync_token = parse(r_one, 'SYNCHRONIZER_TOKEN')
    sync_uri = parse(r_one, 'SYNCHRONIZER_URI')

    # Sign in to the initial login page
    headers_one = {
        'Host': 'aggietime.usu.edu',
        'Content-Type': 'application/x-www-form-urlencoded',
        'Origin': 'https://aggietime.usu.edu',
        'Accept-Encoding': 'gzip, deflate, br',
        'Connection': 'keep-alive',
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.2 Safari/605.1.15',
        'Referer': 'https://aggietime.usu.edu/login/auth',
        'Content-Length': '150',
        'Accept-Language': 'en-us',
    }

    r_two = s.post("https://aggietime.usu.edu/j_spring_security_check",
                   data=f"SYNCHRONIZER_TOKEN={sync_token}&SYNCHRONIZER_URI={sync_uri}&j_username={username}&j_password={password}&login-submit=", headers=headers_one)
    if r_two.status_code == 200:
        print('Succesful login!\n')
    else:
        print("failure at login\n")
    return r_two.status_code


def get_dashboard():
    # Get the dashboard page that has the next sync_token
    data = {}
    r = s.get("https://aggietime.usu.edu/dashboard")
    print(r.status_code)
    if r.status_code == 200:
        print('Success at getting dashboard!')
    else:
        print("failure getting dashboard")
    data['sync_token'] = parse(r, 'SYNCHRONIZER_TOKEN')
    data['sync_uri'] = parse(r, 'SYNCHRONIZER_URI')
    data['posId'] = parse(r, 'posId')
    data['toStatus'] = parse(r, 'toStatus')
    return data


def clock_status():
    data = get_dashboard()
    if data['toStatus'] == "OUT":
        return "You are clocked in"
    else:
        return "You are clocked out"


def clock(inout: str):

    data = get_dashboard()

    headers_two = {
        'Host': 'aggietime.usu.edu',
        'Content-Type': 'application/x-www-form-urlencoded',
        'Origin': 'https://aggietime.usu.edu',
        'Accept-Encoding': 'gzip, deflate, br',
        'Connection': 'keep-alive',
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.2 Safari/605.1.15',
        'Referer': 'https://aggietime.usu.edu/dashboard',
        'Content-Length': '189',
        'Accept-Language': 'en-us',
    }
    r = s.post("https://aggietime.usu.edu/dashboard/clock/punch",
               data=f"SYNCHRONIZER_TOKEN={data['sync_token']}&SYNCHRONIZER_URI={data['sync_uri']}&posId={data['posId']}&comment=&projectName=&toStatus={inout}", headers=headers_two)
    if r.status_code == 200:
        print('Success punching clock!')
    elif r.status_code == 500:
        print("Error 500. You probably clocked out when you need to clock in")
    else:
        print("Failure punching clock")
    return r.status_code


app = Flask(__name__)


@app.route('/aggietime/status/', methods=['GET', 'POST'])
def status():
    if request.method == 'POST':
        username: str = str(request.form.get('username'))
        print(username)
        password: str = str(request.form.get('password'))
        login(username, password)
        status = clock_status()
        return status
    return "Please send a form post request"


@app.route('/aggietime', methods=['GET', 'POST'])
def main():
    if request.method == 'POST':
        username: str = str(request.form.get('username'))
        password: str = str(request.form.get('password'))
        inout: str = str(request.form.get('inout'))
        login(username, password)
        r = clock(inout)
        return str(r)
    return "Please send a form POST request with your login information"


if __name__ == "__main__":
    s = requests.Session()
    context = ('/etc/letsencrypt/live/jrmoulton.com-0001/fullchain.pem',
               '/etc/letsencrypt/live/jrmoulton.com-0001/privkey.pem')
    app.run()
