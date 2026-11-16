/** The public plan catalog as Ardoise answers it, cut down to what the landing reads. */
export const CATALOG = {
	plans: [
		{
			key: 'solo',
			name: 'Solo',
			features: {
				'ai.access': false
			},
			prices: []
		},
		{
			key: 'pro',
			name: 'Pro',
			features: {
				'ai.access': true
			},
			prices: [
				{
					interval: 'month',
					amount: 900,
					currency: 'eur',
					lookup_key: 'plan_pro_month'
				},
				{
					interval: 'year',
					amount: 9000,
					currency: 'eur',
					lookup_key: 'plan_pro_year'
				}
			]
		},
		{
			key: 'studio',
			name: 'Studio',
			features: {
				'ai.access': true
			},
			prices: [
				{
					interval: 'month',
					amount: 2900,
					currency: 'eur',
					lookup_key: 'plan_studio_month'
				},
				{
					interval: 'year',
					amount: 29000,
					currency: 'eur',
					lookup_key: 'plan_studio_year'
				}
			]
		}
	]
};
