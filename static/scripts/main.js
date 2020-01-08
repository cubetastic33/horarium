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
	$('#showOldTimetables').prop('checked', localStorage.getItem('showOldTimetables') === 'true');
	$('#showSubjectNames').prop('checked', localStorage.getItem('showSubjectNames') === 'true');
	$('#time24h').prop('checked', localStorage.getItem('time24h') === 'true');
	$('.overlay').show();
	$('#settingsDialog').show('slow');
});

$('#cancelSettingsChange').click(function () {
	$('#settingsDialog').hide('slow', function () { $('.overlay').hide() });
});

// Update the site to the current settings
$(document).ready(function () {
	if (localStorage.getItem('darkMode') === null) {
		localStorage.setItem('darkMode', true);
	}
	if (localStorage.getItem('showOldTimetables') === null) {
		localStorage.setItem('showOldTimetables', false);
	}
	if (localStorage.getItem('showSubjectNames') === null) {
		localStorage.setItem('showSubjectNames', false);
	}
	if (localStorage.getItem('time24h') === null) {
		localStorage.setItem('time24h', true);
	}
	$('#darkMode').prop('checked', localStorage.getItem('darkMode') === 'true');
	$('#showOldTimetables').prop('checked', localStorage.getItem('showOldTimetables') === 'true');
	$('#showSubjectNames').prop('checked', localStorage.getItem('showSubjectNames') === 'true');
	$('#time24h').prop('checked', localStorage.getItem('time24h') === 'true');

	if (localStorage.getItem('darkMode') === 'false') {
		$('body').attr('id', 'light');
	}
	if (localStorage.getItem('showOldTimetables') === 'false') {
		$('#timetables > li').each(function() {
			if (Date.parse($(this).children('div').text()) < new Date(+new Date() - 86400000)) {
				$(this).hide();
			}
		});
	}
	if (localStorage.getItem('showSubjectNames') === 'true') {
		$('#timetables > li ul li').addClass('showSubjectNames');
	}
	if (localStorage.getItem('time24h') === 'true') {
		$('#timetables > li ul li').addClass('showTimeFor24H');
	}
});

$('#settingsDialog').submit(function (e) {
	e.preventDefault();
	localStorage.setItem('darkMode', $('#darkMode').is(':checked'));
	localStorage.setItem('showOldTimetables', $('#showOldTimetables').is(':checked'));
	localStorage.setItem('showSubjectNames', $('#showSubjectNames').is(':checked'));
	localStorage.setItem('time24h', $('#time24h').is(':checked'));
	if (!$('#darkMode').is(':checked')) {
		$('body').attr('id', 'light');
	} else {
		$('body').attr('id', '');
	}
	if ($('#showOldTimetables').is(':checked')) {
		$('#timetables > li').show();
	} else {
		$('#timetables > li').each(function() {
			if (Date.parse($(this).children('div').text()) < new Date(+new Date() - 86400000)) {
				$(this).hide();
			}
		});
	}
	if ($('#showSubjectNames').is(':checked')) {
		$('#timetables > li ul li').addClass('showSubjectNames');
	} else {
		$('#timetables > li ul li').removeClass('showSubjectNames');
	}
	if ($('#time24h').is(':checked')) {
		$('#timetables > li ul li').addClass('showTimeFor24H');
	} else {
		$('#timetables > li ul li').removeClass('showTimeFor24H');
	}
	$('#settingsDialog').hide('slow', function () { $('.overlay').hide() });
});
