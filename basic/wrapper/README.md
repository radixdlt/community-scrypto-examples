# Wrapper/Bridge

-    This code example was made to find out how one dApp could have access to another dApp's method that requires a badge for System Level Authentication.</br>

</br>
The following step are needed:

- The dApp needs a vault for the authentication badge containing the badge.
- The dApp needs provide the proof when making the call to the service.

The use-case solution was a lot easier then expected.</br> It can actualy be done in a single line when the service_vault contains the required badge:

        let result = self.service_vault.authorize(||component.call::<i8>("awesome_service", args![]));

## Getting Started
-   Source the sourceme on Linux/Bash for an easy start.

        %/> source sourceme

## <b>Wrapper dApp methods:</b>

### <b>simple_bridge</b>
This function will make a call to a external dApp method that does not require authentication.</br>
This call does require Application Level Authentication</br>

### <b>restricted_bridge</b>
This function will make a call to a external dApp method that does require authentication.</br>
This call does require Application Level Authentication</br>

### <b>deposit_badge</b>
Method to deposit the badge required for access to the external dApp</br>

### <b>retreive_badge</b>
Method to retreive the badge required for access to the external dApp</br>
This method requires Admin Badge Sytem Level Authentication</br> 

### <b>withdrawal_all</b>
Method to withdrawal all the XRD accumulated by selling subscription badges.</br>
This method requires Admin Badge Sytem Level Authentication</br> 

### <b>buy_subscription_ticket</b>
Method to buy a subscription ticket to get access to the Wrapper/Bridge component</br>

### <b>admin_subscription_ticket</b>
Method to obtain a free subscription ticket to get access to the Wrapper/Bridge component</br>
This method requires Admin Badge Sytem Level Authentication</br> 

### <b>burn_subscription_ticket</b>
Method to burn a (expired) subscription ticket. 

## <b>Service dApp methods:</b>
### <b>awesome_service</b>
A awesome service, resticted by Sytem Level Authentication</br>
### <b>deep_thought</b>
A less awesome service, free public access.</br>         
        

