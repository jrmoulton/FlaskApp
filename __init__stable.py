from flask import Flask
from flask import request
import requests
from bs4 import BeautifulSoup as bs

app = Flask(__name__)

logged_in = False


def parse(r, id):
    text = r.text
    soup = bs(text, "html.parser")
    line = soup.find(id=id)
    print(line)
    return line.get("value")


def login(username, password, s):
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
    length = str(len(
        f"SYNCHRONIZER_TOKEN={sync_token}&SYNCHRONIZER_URI={sync_uri}&j_username={username}&j_password={password}&login-submit="))
    login_page_headers = {
        'Host': 'aggietime.usu.edu',
        'Content-Type': 'application/x-www-form-urlencoded',
        'Origin': 'https://aggietime.usu.edu',
        'Accept-Encoding': 'gzip, deflate, br',
        'Connection': 'keep-alive',
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.2 Safari/605.1.15',
        'Referer': 'https://aggietime.usu.edu/login/auth',
        'Content-Length': length,
        'Accept-Language': 'en-us',
    }

    r_two = s.post("https://aggietime.usu.edu/j_spring_security_check",
                   data=f"SYNCHRONIZER_TOKEN={sync_token}&SYNCHRONIZER_URI={sync_uri}&j_username={username}&j_password={password}&login-submit=", headers=login_page_headers)
    if r_two.status_code == 200:
        return 'Succesful login!\n'
    else:
        return "Failure logging in\n"


def get_dashboard(username, password, s):

    if not logged_in:
        login(username, password, s)

    # Get the dashboard page that has the next sync_token
    variables = {}
    print(s.cookies)
    r = s.get("https://aggietime.usu.edu/dashboard")
    print(r.status_code)
    if r.status_code == 200:
        print('Success at getting dashboard!')
    else:
        print("failure getting dashboard")
    variables['sync_token'] = parse(r, 'SYNCHRONIZER_TOKEN')
    variables['sync_uri'] = parse(r, 'SYNCHRONIZER_URI')
    variables['toStatus'] = parse(r, 'toStatus')
    variables['posId'] = parse(r, 'posId')

    return variables


def punch_clock(username, password, inout, s):

    variables = get_dashboard(username, password, s)

    length = str(len(
        f"SYNCHRONIZER_TOKEN={variables['sync_token']}&SYNCHRONIZER_URI={variables['sync_uri']}&posId={variables['posId']}&comment=&projectName=&toStatus={inout}"))
    clock_punch_headers = {
        'Host': 'aggietime.usu.edu',
        'Content-Type': 'application/x-www-form-urlencoded',
        'Origin': 'https://aggietime.usu.edu',
        'Accept-Encoding': 'gzip, deflate, br',
        'Connection': 'keep-alive',
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.2 Safari/605.1.15',
        'Referer': 'https://aggietime.usu.edu/dashboard',
        'Content-Length': length,
        'Accept-Language': 'en-us',
    }
    r = s.post("https://aggietime.usu.edu/dashboard/clock/punch",
               data=f"SYNCHRONIZER_TOKEN={variables['sync_token']}&SYNCHRONIZER_URI={variables['sync_uri']}&posId={variables['posId']}&comment=&projectName=&toStatus={inout}", headers=clock_punch_headers)
    if r.status_code == 200:
        if variables['toStatus'] == 'OUT':
            return 'Clocked Out'
        else:
            return 'Clocked In'
    elif r.status_code == 500:
        return "Error 500. You probably clocked out when you need to clock in"
    else:
        return "Failure punching clock"


def clock_status(username, password, s):
    variables = get_dashboard(username, password, s)
    if variables['toStatus'] == "OUT":
        return "You're clocked in"
    else:
        return "You're clocked out"


@app.route('/aggietime/', methods=['GET', 'POST'])
def main():
    s = requests.Session()
    if request.method == 'POST':
        username: str = str(request.form.get('username'))
        password: str = str(request.form.get('password'))
        inout: str = str(request.form.get('inout'))
        r = punch_clock(username, password, inout, s)
        return r
    return "Please send a form POST request with your login information"


@app.route('/aggietime/status/', methods=['GET', 'POST'])
def status():
    s = requests.Session()
    if request.method == 'POST':
        username: str = str(request.form.get('username'))
        password: str = str(request.form.get('password'))
        status_code = clock_status(username, password, s)
        return status_code
    return "Please send a form post request"


if __name__ == "__main__":
    context = ('/etc/letsencrypt/live/jrmoulton.com/fullchain.pem',
               '/etc/letsencrypt/live/jrmoulton.com//privkey.pem')
    app.run(debug=True, ssl_context=context, host='jrmoulton.com')
    # app.run()
