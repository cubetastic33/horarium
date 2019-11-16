# horarium
A website to view class timetables

# FAQ

## How does this work?

This website gets the timetables from FIITJEE's servers, scrapes the data from the HTML, then stores it in its own database so that it function faster on later loads. This website is open source, and you can view the source code right here, in this GitHub repo.

## Can I install this like an app?

Yes. This website is a Progressive Web App (PWA), which means you can install and use this as an app. Visiting the site on supported browsers should show a banner to add it to your home screen. Click [https://horarium.herokuapp.com/install_pwa](here) for detailed instructions.

## Why do the timetable pages occasionally take so long to load?
Depending on your network connection, if the pages take more than 5 seconds to load, it's probably because it's trying to get new timetables from FIITJEE. Whenever the site finds that it doesn't have the timetables you're asking for in its database, it goes over to FIITJEE's servers to get them. This means that most of the time this site should work fast, and only occasionally will it take time.

## What languages is this written in?

The backend is written in rust, it uses a PostgreSQL database, and the frontend is written in HTML, SASS (compiled to CSS), and JavaScript.

## How do I contact you?

Feedback is very much appreciated. I would love to hear your feedback. If you have any, or want to ask something, contact me [here](mailto:aravk33@pm.me).

# Screenshots
![Home Page](https://i.imgur.com/eEl3ddA.png)

![Timetables Page](https://i.imgur.com/BwTxWMI.png)

![Logs Page](https://i.imgur.com/J9FuxB8.png)
