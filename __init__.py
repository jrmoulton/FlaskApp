from flask import Flask
from flask import request
import requests
from bs4 import BeautifulSoup as bs
from typing import List, Any, Tuple, Dict
import re

app = Flask(__name__)

logged_in = False


def parse_id(text: str, id: str) -> str:
    soup = bs(text, "html.parser")
    line = soup.find(id=id)
    return line.get("value")


def parse_all_id(r, id: str) -> List[str]:
    ids: List[str] = []
    text = r.text
    soup = bs(text, "html.parser")
    lines = soup.find_all(id=id)
    for count, id_line in enumerate(lines):
        ids[count] = parse_id(id_line, id)
    return ids


def parse_one_id(r, id: str) -> str:
    return parse_all_id(r, id)[0]


def login(username, password, s) -> str:
    # Get the initial login page with the sync token
    r_one = s.get('https://aggietime.usu.edu/login/auth')
    print(r_one.status_code)
    if r_one.status_code == 200:
        print('Succesful initial!')
    else:
        print("failure at initial")
    sync_token = parse_id(r_one, 'SYNCHRONIZER_TOKEN')
    sync_uri = parse_id(r_one, 'SYNCHRONIZER_URI')

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


def get_dashboard(s) -> Tuple[Dict[str, str], Any]:

    # Get the dashboard page that has the next sync_token
    variables: Dict[str, str] = {}
    print(s.cookies)
    r = s.get("https://aggietime.usu.edu/dashboard")
    print(r.status_code)
    if r.status_code == 200:
        print('Success at getting dashboard!')
    else:
        print("failure getting dashboard")
    variables['sync_token'] = parse_id(r, 'SYNCHRONIZER_TOKEN')
    variables['sync_uri'] = parse_id(r, 'SYNCHRONIZER_URI')
    variables['toStatus'] = parse_id(r, 'toStatus')
    variables['posId'] = parse_id(r, 'posId')

    return (variables, r)


def punch_clock(inout: str, s) -> str:

    variables, _ = get_dashboard(s)

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
            return 'Success clocking out'
        else:
            return 'Success clocking in'
    elif r.status_code == 500:
        return "Error 500. You probably clocked out when you need to clock in"
    else:
        return "Failure punching clock"


def clock_status(s) -> str:
    variables, _ = get_dashboard(s)
    if variables['toStatus'] == "OUT":
        return "You are clocked in"
    else:
        return "You are clocked out"


def return_last_shift(s):
    variables, r = get_dashboard(s)
    pass


def edit_shift(id, time_in, time_out):
    pass


@app.route('/aggietime/', methods=['GET', 'POST'])
def main_clock() -> str:
    if request.method == 'POST':
        s = requests.Session()
        username: str = str(request.form.get('username'))
        password: str = str(request.form.get('password'))
        inout: str = str(request.form.get('inout'))
        if not logged_in:
            login(username, password, s)
        r = punch_clock(inout, s)
        return r
    return "Please send a form POST request with your login information"


@app.route('/aggietime/status/', methods=['GET', 'POST'])
def main_status() -> str:
    if request.method == 'POST':
        s = requests.Session()
        username: str = str(request.form.get('username'))
        password: str = str(request.form.get('password'))
        if not logged_in:
            login(username, password, s)
        status_code = clock_status(s)
        return status_code
    return "Please send a form post request"


@app.route('/aggietime/check_positions/', methods=['GET', 'POST'])
def get_positions(s):
    if request.method == 'POST':
        s = requests.Session()
        username: str = str(request.form.get('username'))
        password: str = str(request.form.get('password'))
        if not logged_in:
            login(username, password, s)
        _, r = get_dashboard(s)
        # Match up to and including the '(' char
        find = re.compile(r".*\(")
        return_string: str = ""
        positions: List[Tuple[str, str]] = []
        text = r.text
        soup = bs(text, "html.parser")
        tags = soup.find_all(tag="option")
        for count, tag in enumerate(tags):
            positions[count] = (tag.get("value"), tag.contents)

        for index in range(len(positions)):
            name = positions[index][1]
            name = re.search(find, name).group(0)[:-1]
            return_string += f"{positions[index][0]}: {name}\n"
        return return_string
    else:
        return "Please send a form post request"


if __name__ == "__main__":
    context = ('/etc/letsencrypt/live/jrmoulton.com/fullchain.pem',
               '/etc/letsencrypt/live/jrmoulton.com//privkey.pem')
    app.run(debug=True, ssl_context=context, host='jrmoulton.com')
    # app.run()
