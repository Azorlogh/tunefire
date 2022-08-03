INSERT INTO songs
VALUES
	(
		'03f221f0-ff57-4c56-ad8c-e8e7bec996a0',
		'thook - RUDE',
		'https://soundcloud.com/thook/rude'
	),
	(
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a1',
		'thook - SEE THRU',
		'https://soundcloud.com/thook/see-thru'
	),
	(
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a2',
		'L*o*J - Medusa In Naboo',
		'https://soundcloud.com/loj-2/medusa-in-naboo'
	),
	(
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a3',
		'chromonicci - Cerulean.',
		'https://soundcloud.com/phuturecollective/chromonicci-cerulean'
	);

INSERT INTO tags
VALUES
	(
		"0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a4",
		"harsh"
	),
	(
		"0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a5",
		"bouncy"
	),
	(
		"0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a6",
		"regular"
	);

INSERT INTO song_tags
VALUES
	(
		'03f221f0-ff57-4c56-ad8c-e8e7bec996a0',
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a4',
		0.6
	),
	(
		'03f221f0-ff57-4c56-ad8c-e8e7bec996a0',
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a5',
		0.5
	),
	(
		'03f221f0-ff57-4c56-ad8c-e8e7bec996a0',
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a6',
		0.5
	),

	(
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a1',
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a4',
		0.75
	),
	(
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a1',
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a5',
		0.25
	),
	(
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a1',
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a6',
		0.75
	),

	(
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a2',
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a4',
		0.5
	),
	(
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a2',
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a5',
		0.85
	),
	(
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a2',
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a6',
		0.4
	),

	(
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a3',
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a4',
		0.1
	),
	(
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a3',
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a5',
		0.9
	),
	(
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a3',
		'0c8b8f0c-f8f0-4c9f-b8e8-e8e7bec996a6',
		0.8
	);
