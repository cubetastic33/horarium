$('#timetables > li').click(function(e) {
	if (!$(this).hasClass('open')) {
		// We're opening a timetable, so we should close all the other timetables
		$('li.open ul').hide('blind', () => $('li.open').toggleClass('open'));
	}
	if (!$(this).hasClass('open') || ($(this).hasClass('open') && this.children[0].contains(e.target))) {
		$(this).children('ul').toggle('blind', () => $(this).toggleClass('open'));
	}
});

$('button').click(() => {
	// Forcibly refetch timetables
	$('button').attr('disabled', true);
	showToast('Please wait...', 10000);
	$.ajax({
		type: 'POST',
		url: '/refetch/' + window.location.pathname.split('/')[1],
		success: result => {
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
