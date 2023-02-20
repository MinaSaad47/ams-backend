# admins

- [x] /api/admins/login (POST) `login`
- [x] /api/subjects/ (GET | POST) `get, create a subject`
- [x] /api/subjects/<id> (GET | UPDATE | DELETE) `get all, update, delete subject(s)`
- [x] /api/instructors/ (GET | POST) `get, create an instructor(s)`
- [x] /api/instructors/<id> (GET | UPDATE | DELETE) `get, update, delete an instructor`
- [x] /api/attendees/ (GET | POST) `get, create an attendee`
- [x] /api/attendees/<id> (GET | UPDATE | DELETE) `get, update, delete an attendee`

# instructors

- [x] /api/instructors/login (POST) `login`
- [x] /api/instructors/<id> (GET) `view profile`
- [x] /api/instructors/<id>/subjects/ (GET) `view subjects`
- [x] /api/instructors/<id>/subjects/<id> (GET | PUT | DELETE) `view, add, delete a subject`
- [-] /api/attendances/subjects/<id> (GET) `view attendances`
- [-] /api/attendances/subjects/<id>/attendees/<id>?method=<id|qr|face> (PUT) `take attendance`

# attendees

- [x] /api/attendees/login (POST) `login`
- [x] /api/attendees/<id> (GET) `view profile`
- [-] /api/attendees/<id>/subjects/ (GET) `view subjects`
- [-] /api/attendees/<id>/subjects/<id> (GET | PUT) `view, add a subject`
- [-] /api/attendees/<id>/subjects/<id>/attendances (GET) `view attendances`
