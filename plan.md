
1. Get the initial login page - https://aggietime.usu.edu/login/auth - no cookies needed

2. Parse the sync token

3. Send the auth_login with the sync token - 

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
