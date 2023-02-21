CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE TABLE IF NOT EXISTS admins (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(32) NOT NULL,
    email VARCHAR(32) NOT NULL CONSTRAINT admin_email_must_be_unique UNIQUE,
    password VARCHAR NOT NULL,
    create_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS attendees (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    number BIGINT NOT NULL CONSTRAINT attendee_number_must_be_unique UNIQUE,
    name VARCHAR(32) NOT NULL,
    email VARCHAR(32) NOT NULL CONSTRAINT attendee_email_must_be_unique UNIQUE,
    password VARCHAR NOT NULL,
    embedding DOUBLE PRECISION [],
    create_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS instructors (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    number BIGINT NOT NULL CONSTRAINT instructor_number_must_be_unique UNIQUE,
    name VARCHAR(32) NOT NULL,
    email VARCHAR(32) NOT NULL CONSTRAINT instructor_email_must_be_unique UNIQUE,
    password VARCHAR NOT NULL,
    create_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS subjects (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(32) NOT NULL CONSTRAINT subject_must_be_unique UNIQUE,
    instructor_id UUID REFERENCES instructors(id),
    cron_expr VARCHAR NOT NULL,
    create_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS attendees_subjects (
    attendee_id UUID REFERENCES attendees(id) ON DELETE CASCADE,
    subject_id UUID REFERENCES subjects(id) ON DELETE CASCADE,
    CONSTRAINT attendees_subjects_pkey PRIMARY KEY (attendee_id, subject_id)
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