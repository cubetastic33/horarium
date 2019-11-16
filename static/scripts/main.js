// A function to show a toast
function showToast(message, duration = 2000) {
	$('#toast').text(message);
	$('#toast').slideDown(200, function () {
		setTimeout(function () {
			$('#toast').slideUp(200);
		}, duration);
	});
}

// Show installation dialog on iOS if opened normally and we haven't shown the dialog before
if (!window.matchMedia('(display-mode: standalone)').matches && localStorage['iOSShownInstallationDialog'] === undefined && navigator.platform && /iPad|iPhone/.test(navigator.platform)) {
	$('.overlay').show();
	$('#iOSInstallPwa').show('slow');
	localStorage.setItem('iOSShownInstallationDialog', Date.now());
	$('#closeiOSInstallPwaDialog').click(function () {
		$('#iOSInstallPwa').hide('slow', function () { $('.overlay').hide() });
	});
}

// Settings
$('#settingsButton').click(function () {
	$('#darkMode').prop('checked', localStorage.getItem('darkMode') === 'true');
	$('#showSubjectNames').prop('checked', localStorage.getItem('showSubjectNames') === 'true');
	$('.overlay').show();
	$('#settingsDialog').show('slow');
});

$('#cancelSettingsChange').click(function () {
	$('#settingsDialog').hide('slow', function () { $('.overlay').hide() });
});

// Update the site to the current settings
$(document).ready(function () {
	if (localStorage.getItem('darkMode') === null || localStorage.getItem('showSubjectNames') === null) {
		localStorage.setItem('darkMode', true);
		localStorage.setItem('showSubjectNames', false);
	}
	$('#darkMode').prop('checked', localStorage.getItem('darkMode') === 'true');
	$('#showSubjectNames').prop('checked', localStorage.getItem('showSubjectNames') === 'true');

	if (localStorage.getItem('darkMode') === 'false') {
		$('body').attr('id', 'light');
	}
	if (localStorage.getItem('showSubjectNames') === 'true') {
		$('#timetables > li ul li').addClass('class', 'showSubjectNames');
	}
});

$('#settingsDialog').submit(function (e) {
	e.preventDefault();
	localStorage.setItem('darkMode', $('#darkMode').is(':checked'));
	localStorage.setItem('showSubjectNames', $('#showSubjectNames').is(':checked'));
	if (!$('#darkMode').is(':checked')) {
		$('body').attr('id', 'light');
	} else {
		$('body').attr('id', '');
	}
	if ($('#showSubjectNames').is(':checked')) {
		$('#timetables > li ul li').addClass('showSubjectNames');
	} else {
		$('#timetables > li ul li').removeClass('showSubjectNames');
	}
	$('#settingsDialog').hide('slow', function () { $('.overlay').hide() });
});
