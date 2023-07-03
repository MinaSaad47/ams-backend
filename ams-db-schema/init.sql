CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS admins (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(32) NOT NULL,
    email VARCHAR(32) NOT NULL CONSTRAINT uk_admin_email_must_be_unique UNIQUE,
    password VARCHAR NOT NULL,
    create_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS attendees (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    number BIGINT NOT NULL CONSTRAINT uk_attendee_number_must_be_unique UNIQUE,
    name VARCHAR(32) NOT NULL,
    email VARCHAR(32) NOT NULL CONSTRAINT uk_attendee_email_must_be_unique UNIQUE,
    password VARCHAR NOT NULL,
    embedding DOUBLE PRECISION [],
    image VARCHAR(256),
    create_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS instructors (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    number BIGINT NOT NULL CONSTRAINT uk_instructor_number_must_be_unique UNIQUE,
    name VARCHAR(32) NOT NULL,
    email VARCHAR(32) NOT NULL CONSTRAINT uk_instructor_email_must_be_unique UNIQUE,
    password VARCHAR NOT NULL,
    image VARCHAR(256),
    create_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS subjects (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(32) NOT NULL CONSTRAINT uk_subject_must_be_unique UNIQUE,
    instructor_id UUID REFERENCES instructors(id),
    create_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS subject_dates (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    day_of_week INTEGER NOT NULL,
    start_time TIME WITHOUT TIME ZONE NOT NULL,
    end_time TIME WITHOUT TIME ZONE NOT NULL,
    subject_id UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    create_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT ck_day_of_week CHECK (day_of_week >= 0 AND day_of_week <= 6),
    CONSTRAINT ck_start_end_time CHECK (start_time < end_time)
);

CREATE TABLE IF NOT EXISTS attendees_subjects (
    attendee_id UUID REFERENCES attendees(id) ON DELETE CASCADE,
    subject_id UUID REFERENCES subjects(id) ON DELETE CASCADE,
    CONSTRAINT pk_attendees_subjects_pkey PRIMARY KEY (attendee_id, subject_id)
);

CREATE TABLE IF NOT EXISTS attendances (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id UUID NOT NULL REFERENCES subjects(id),
    attendee_id UUID NOT NULL REFERENCES attendees(id),
    create_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO admins (name, email, password)
VALUES ('Mina Saad', 'mina@saad.com', '474747'),
    ('Mina Emil', 'mina@emil.com', '393939'),
    ('Mina Girgs', 'mina@girgs.com', '717171');
