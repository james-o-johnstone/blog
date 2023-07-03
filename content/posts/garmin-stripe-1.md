---
title: "Stripe Billing Integration for Garmin (Connect IQ) Apps: Part 1"
date: 2023-07-02T16:59:46+01:00
draft: false
---

# Overview
Garmin does not provide any payment handling for Connect IQ apps which means that the app developer is required to implement their own payment solution. 

Common solutions to the problem use activation codes, a user pays the app developer using Paypal, the app developer emails a notification code and the user must enter this manually. This is time consuming, error prone and does not provide a great user experience. 

To improve this experience, I decided to integrate [Humin](https://apps.garmin.com/en-US/apps/5ed9382a-6f47-419d-a21c-fb72b725842b) with Stripe to automate the product activation process. At the end of the trial period, the user receives a [payment link](https://stripe.com/docs/payment-links) and after successful payment, a [webhook](https://stripe.com/docs/webhooks) updates the paid status of the user.

This post will explain the main concepts and examples to get this flow working. There is also a full example [repo](https://github.com/james-o-johnstone/garmin-stripe) containing the backend code. In order to use this solution you will also need to host your own webserver. Humin's backend is hosted on a [Hetzner](https://www.hetzner.com/cloud) CAX11 which provides a cheap but plenty powerful server for this usecase. In part 2 I will describe the deployment process and provide yaml files to bootstrap a server and deploy the backend.

The main concept underpinning this approach is the [unique identifier](https://developer.garmin.com/connect-iq/api-docs/Toybox/System/DeviceSettings.html#uniqueIdentifier-var) that Garmin makes accessible through the SDK.

# Unique Identifier
When a user first starts up the app on Garmin, a unique identifier is created which is unique for every app, but is stable on a device across uninstall and reinstall.

The unique identifier will be sent to the web app on first open, and will be stored in a database alongside the creation time, marking the start of the trial period (I use sqlite for a DB, so nothing too fancy/complicated).

Each time the app is opened on the device, a request is sent to the webserver to check if the user's trial has expired. After the trial period is up, when the user next opens the app, Humin will open a stripe payment link on their smartphone browser using the [`openWebPage`](https://developer.garmin.com/connect-iq/api-docs/Toybox/Communications.html#openWebPage-instance_function) function.

Below is some example code showing how to obtain the unique identifier and send it to the backend, along with checking the paid status of the user in the response. If the user's trial has expired then we show the payment link, otherwise they can continue to use the app. The payment link is also returned from the backend, rather than being hardcoded in the app so it is easier to modify if needed (just redeploy the web app rather than rolling out an update through Connect IQ).

One thing to note here is that when displaying the payment link, we will set a [url parameter](https://stripe.com/docs/payment-links/url-parameters#streamline-reconciliation-with-a-url-parameter) called `client_reference_id` equal to the unique identifier. This parameter will be sent alongside any webhooks relating to the payment, so we can use it in the webhook handler to associate the webhook with the user.

```
using Toybox.Communications as Comm;
using Toybox.Authentication as Auth;
using Toybox.System;

class UserManager {
  private var ID = System.getDeviceSettings().uniqueIdentifier;

  function newUser() {
    var params = {
      "id" => ID
    };
    var options = {
      :method => Comm.HTTP_REQUEST_METHOD_POST,
      :contentType => Comm.REQUEST_CONTENT_TYPE_JSON,
      :headers => {
        "api-key" => $.API_KEY
        }
      };
    Comm.makeWebRequest($.BASE_URL + "/user", params, options, method(:onResponse));
  }

  private function onResponse(responseCode, data) {
    if (responseCode != 200) {
      return handleError(responseCode, data);
    }

    var status = data["paid_status"];
    if (!status.equals("paid") && !status.equals("trial")) {
      var params = {
        "client_reference_id" => ID
      };
      Comm.openWebPage(data["payment_link"], params, null);
    } else {
      Ui.switchToView(new MainView(), new MainViewDelegate(), Ui.SLIDE_IMMEDIATE);
    }
  }
}
```

The backend code that handles the `/user` request can be found [here]().

# Webhook Handler

Stripe provides good [documentation](https://stripe.com/docs/webhooks) on how to implement webhooks in your language of choice. We are only interested in the [`checkout.session.completed`](https://stripe.com/docs/api/events/types#event_types-checkout.session.completed) event type, because when we receive this event we know the user has paid to unlock the application so we can update their `paid_status`. When we handle the webhook, we can grab the `ClientReferenceID` from the [`CheckoutSession`](https://stripe.com/docs/api/checkout/sessions/object#checkout_session_object-client_reference_id) which allows us to associate the webhook to the Garmin unique identifier and update the correct user.

Here is a cut down example for handling the webhook and updating the user, the full backend code can be found in the [repo](https://github.com/james-o-johnstone/garmin-stripe). 

```
func handleWebhook(w http.ResponseWriter, req *http.Request) {
  // Parse the request and verify webhook signature
  // https://stripe.com/docs/webhooks/signatures

  switch event.Type {
    case "checkout.session.completed":
      var cs stripe.CheckoutSession
      err := json.Unmarshal(event.Data.Raw, &cs)
      if err != nil {
      	fmt.Fprintf(os.Stderr, "Error parsing webhook JSON: %v\n", err)
      	w.WriteHeader(http.StatusBadRequest)
      	return
      }
      if cs.ClientReferenceID == "" {
      	w.WriteHeader(http.StatusBadRequest)
      	return
      }
      success := db.SetPaid(cs.ClientReferenceID)
      if !success {
      	fmt.Fprintf(os.Stderr, "Checkout session failed for: %s\n", cs.ClientReferenceID)
      	w.WriteHeader(http.StatusInternalServerError)
      	return
      }
      fmt.Fprintf(os.Stdout, "Checkout session completed for: %s\n", cs.ClientReferenceID)

    default:
      // unhandled
  }
  w.WriteHeader(http.StatusOK)
}
