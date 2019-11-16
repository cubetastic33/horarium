$('#timetables > li').click(function (e) {
	var element = this;
	if (!$(element).hasClass('open')) {
		// We're opening a timetable, so we should close all the other timetables
		$('li.open ul').hide('blind', function () { $('li.open').toggleClass('open') });
	}
	if (!$(element).hasClass('open') || ($(element).hasClass('open') && element.children[0].contains(e.target))) {
		$(element).children('ul').toggle('blind', function () { $(element).toggleClass('open') });
	}
});

$('#forciblyRefetch').click(function () {
	// Forcibly refetch timetables
	$('button').attr('disabled', true);
	showToast('Please wait...', 10000);
	$.ajax({
		type: 'POST',
		url: '/refetch/' + window.location.pathname.split('/')[1],
		success: function(result) {
			console.log(result);
			showToast(result, 10000);
			// If we've successfully refetched we can refresh the page
			if (result === 'success') {
				window.location.reload();
			} else {
				// There was an error, so re-enable the button
				$('button').attr('disabled', false);
			}
		}
	});
});

function urlBase64ToUint8Array(base64String) {
	const padding = '='.repeat((4 - base64String.length % 4) % 4);
	const base64 = (base64String + padding)
		.replace(/\-/g, '+')
		.replace(/_/g, '/');

	const rawData = window.atob(base64);
	const outputArray = new Uint8Array(rawData.length);

	for (let i = 0; i < rawData.length; ++i) {
		outputArray[i] = rawData.charCodeAt(i);
	}
	return outputArray;
}

function askPermission() {
	return new Promise(function(resolve, reject) {
		const permissionResult = Notification.requestPermission(function(result) {
			resolve(result);
		});

		if (permissionResult) {
			permissionResult.then(resolve, reject);
		}
	})
	.then(function(permissionResult) {
		if (permissionResult !== 'granted') {
			throw new Error('We weren\'t granted permission.');
		} else {
			subscribeUserToPush();
		}
	});
}

function subscribeUserToPush() {
	return navigator.serviceWorker.register('/service-worker.js')
		.then(function (registration) {
			const subscribeOptions = {
				userVisibleOnly: true,
				applicationServerKey: urlBase64ToUint8Array(
					'BGsl1j5f19bV38u9Umew3r2ACtDPXLM0M46tonVU1gseOS1Fd_gM2bp0NstcHt0ehIANFrn9q3f50i9meL5r334='
				)
			};

			return registration.pushManager.subscribe(subscribeOptions);
		})
		.then(function (pushSubscription) {
			console.log('Received PushSubscription: ', pushSubscription);
			$.ajax({
				type: 'POST',
				url: 'subscribe_notifications',
				data: JSON.stringify(pushSubscription),
				success: function (result) {
					console.log(result);
					showToast(result);
				}
			});
			return pushSubscription;
		});
}

if ('Notification' in window && (localStorage['lastShownNotificationDialog'] === undefined || ((Date.now() - localStorage['lastShownNotificationDialog']) / 86400000) >= 7)) {
	$('.overlay').show();
	$('#notificationRequestDialog').show('slow');
	$('#acceptNotificationRequest').click(function () {
		askPermission();
		$('#notificationRequestDialog').hide('slow', function () { $('.overlay').hide() });
		localStorage.setItem('lastShownNotificationDialog', Date.now());
	});
	$('#cancelNotificationRequest').click(function () {
		$('#notificationRequestDialog').hide('slow', function () { $('.overlay').hide() });
		localStorage.setItem('lastShownNotificationDialog', Date.now());
	});
}
