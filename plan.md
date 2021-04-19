
1. Get the initial login page - https://aggietime.usu.edu/login/auth - no cookies needed

2. Parse the sync token

3. Send the auth_login with the sync token - 
<<<<<<< HEAD
    https://aggietime.usu.edu/j_spring_security_check - SYNCHRONIZER_TOKEN=db388094-cb3b-4e6e-ad1d-ffcdbc67cb82&SYNCHRONIZER_URI=%2Flogin%2Fauth&j_username=A02226665&j_password=&login-submit=
=======
    https://aggietime.usu.edu/j_spring_security_check - SYNCHRONIZER_TOKEN=db388094-cb3b-4e6e-ad1d-ffcdbc67cb82&SYNCHRONIZER_URI=%2Flogin%2Fauth&j_username=A02226665&j_password=E2be2rmamsabite&login-submit=
>>>>>>> ca2a14da4f7ba25e789435667c10cd67e783d1be

4. Get the session id cookie from the response and send it with the request to aggietime.usu.edu/dashboard

5. Parse the sync token again

7. Send the clock in request to https://aggietime.usu.edu/dashboard/clock/punch 
for SYNCHRONIZER_TOKEN=3c424bdf-d07b-4abc-aeff-46a92f2006fd&SYNCHRONIZER_URI=%2Fdashboard&deptText=Campus+Store&posText=Technology+Sales+Associate&posId=155699&comment=&projectName=&toStatus=OUT (or IN)

8. In order to make another request the dashboard has to be requested again with the session id

9. Parse the sync token from that response

10. Send the logout request with the sync token




Login with the client
get dashboard for the SYNCHRONIZER_TOKEN 
send clock punch
