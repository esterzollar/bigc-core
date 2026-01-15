use rand::Rng;
use uuid::Uuid;

#[derive(Clone)]
pub struct BigLuck {
    first_names: Vec<&'static str>,
    last_names: Vec<&'static str>,
    zips: Vec<&'static str>,
    streets: Vec<&'static str>,
    user_agents: Vec<&'static str>,
}

impl BigLuck {
    pub fn new() -> Self {
        BigLuck {
            first_names: vec![
                "James", "Mary", "John", "Patricia", "Robert", "Jennifer", "Michael", "Linda", "William", "Elizabeth",
                "David", "Barbara", "Richard", "Susan", "Joseph", "Jessica", "Thomas", "Sarah", "Charles", "Karen",
                "Christopher", "Nancy", "Daniel", "Lisa", "Matthew", "Betty", "Anthony", "Margaret", "Mark", "Sandra",
                "Donald", "Ashley", "Steven", "Kimberly", "Paul", "Emily", "Andrew", "Donna", "Joshua", "Michelle",
                "Kenneth", "Dorothy", "Kevin", "Carol", "Brian", "Amanda", "George", "Melissa", "Edward", "Deborah",
                "Ronald", "Stephanie", "Timothy", "Rebecca", "Jason", "Sharon", "Jeffrey", "Laura", "Ryan", "Cynthia",
                "Jacob", "Kathleen", "Gary", "Amy", "Nicholas", "Shirley", "Eric", "Angela", "Stephen", "Helen",
                "Jonathan", "Anna", "Larry", "Brenda", "Justin", "Pamela", "Scott", "Nicole", "Brandon", "Emma",
                "Benjamin", "Samantha", "Samuel", "Katherine", "Frank", "Christine", "Gregory", "Debra", "Raymond", "Rachel",
                "Alexander", "Catherine", "Patrick", "Carolyn", "Jack", "Janet", "Dennis", "Ruth", "Jerry", "Maria"
            ],
            last_names: vec![
                "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez",
                "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson", "Thomas", "Taylor", "Moore", "Jackson", "Martin",
                "Lee", "Perez", "Thompson", "White", "Harris", "Sanchez", "Clark", "Ramirez", "Lewis", "Robinson",
                "Walker", "Young", "Allen", "King", "Wright", "Scott", "Torres", "Nguyen", "Hill", "Flores",
                "Green", "Adams", "Nelson", "Baker", "Hall", "Rivera", "Campbell", "Mitchell", "Carter", "Roberts",
                "Gomez", "Phillips", "Evans", "Turner", "Diaz", "Parker", "Cruz", "Edwards", "Collins", "Reyes",
                "Stewart", "Morris", "Morales", "Murphy", "Cook", "Rogers", "Gutierrez", "Ortiz", "Morgan", "Cooper",
                "Peterson", "Bailey", "Reed", "Kelly", "Howard", "Ramos", "Kim", "Cox", "Ward", "Richardson",
                "Watson", "Brooks", "Chavez", "Wood", "James", "Bennett", "Gray", "Mendoza", "Ruiz", "Hughes",
                "Price", "Alvarez", "Castillo", "Sanders", "Patel", "Myers", "Long", "Ross", "Foster", "Jimenez"
            ],
            zips: vec!["90001", "90011", "90026", "90027", "90028", "90029", "90036", "90038", "90045", "90048"],
            streets: vec![
                "AlamedaStreet", "AdamsBoulevard", "AvenueoftheStars", "BeverlyBoulevard", "Broadway", "BundyDrive",
                "CentinelaAvenue", "CentralAvenue", "CesarChavezAvenue", "FairfaxAvenue", "FigueroaStreet", "FlorenceAvenue",
                "FountainAvenue", "GrandAvenue", "HighlandAvenue", "HuntingtonDrive", "ImperialHighway", "JeffersonBoulevard",
                "LaBreaAvenue", "LaCienegaBoulevard", "LaTijeraAvenue", "LaurelCanyonBoulevard", "LincolnBoulevard", "ManchesterAvenue",
                "MartinLutherKingJrBoulevard", "MelroseAvenue", "MissionRoad", "MulhollandDrive", "NormandieAvenue", "ObamaBoulevard",
                "OlympicBoulevard", "PacificCoastHighway", "ParnellStreet", "PicoBoulevard", "ResedaBoulevard", "RobertsonBoulevard",
                "RoscoeBoulevard", "SanVicenteBoulevard", "SantaMonicaBoulevard", "SepulvedaBoulevard", "ShermanWay",
                "SlausonAvenue", "SpringStreet", "SunsetBoulevard", "TopangaCanyonBoulevard", "VanNuysBoulevard", "VeniceBoulevard",
                "VenturaBoulevard", "VermontAvenue", "WashingtonBoulevard", "WilshireBoulevard", "AbbotKinneyBoulevard", "AlvaradoStreet"
            ],
            user_agents: vec![
                "Mozilla/5.0 (Linux; Android 14; 2109119DG Build/UKQ1.240624.001; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/136.0.7103.60 Mobile Safari/537.36",
                "Mozilla/5.0 (Android 14; Mobile; rv:138.0) Gecko/138.0 Firefox/138.0",
                "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Mobile Safari/537.36 EdgA/136.0.0.0",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:139.0) Gecko/20100101 Firefox/139.0",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36 Edg/136.0.0.4",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:134.0) Gecko/20100101 Firefox/134.0",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36 OPR/118.0.0.0",
                "Mozilla/5.0 (Linux; Android 14; Pixel 7 Pro) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.7103.60 Mobile Safari/537.36",
                "Mozilla/5.0 (Linux; Android 14; SM-G998B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.7103.60 Mobile Safari/537.36 SamsungBrowser/23.0",
                "Mozilla/5.0 (Android 14; Mobile; rv:134.0) Gecko/134.0 Firefox/134.0",
                "Mozilla/5.0 (Linux; Android 14; SM-G998B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.7103.60 Mobile Safari/537.36 EdgA/136.0.0.0"
            ],
        }
    }

    pub fn get_first(&self) -> String {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0..self.first_names.len());
        self.first_names[i].to_string()
    }

    pub fn get_last(&self) -> String {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0..self.last_names.len());
        self.last_names[i].to_string()
    }

    pub fn get_zip(&self) -> String {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0..self.zips.len());
        self.zips[i].to_string()
    }

    pub fn get_street(&self) -> String {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0..self.streets.len());
        self.streets[i].to_string()
    }

    pub fn get_user_agent(&self) -> String {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0..self.user_agents.len());
        self.user_agents[i].to_string()
    }

    pub fn get_uuid(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub fn get_email(&self, first: &str, last: &str) -> String {
        let mut rng = rand::thread_rng();
        let num_digits = if rng.gen_bool(0.5) { 2 } else { 4 };
        let rand_num = rng.gen_range(0..10_i32.pow(num_digits));
        let num_str = format!("{:0width$}", rand_num, width = num_digits as usize);

        let provider = if rng.gen_bool(0.33) {
            "gmail"
        } else if rng.gen_bool(0.5) {
            "hotmail"
        } else {
            "outlook"
        };

        // Use either full first or first + last
        if rng.gen_bool(0.5) {
            format!(
                "{}{}{}@{}.com",
                first.to_lowercase(),
                last.to_lowercase(),
                num_str,
                provider
            )
        } else {
            format!(
                "{}{}{}@{}.com",
                first.to_lowercase(),
                num_str,
                num_str,
                provider
            )
        }
    }

    pub fn get_random_num(&self, min: i32, max: i32) -> String {
        let mut rng = rand::thread_rng();
        rng.gen_range(min..=max).to_string()
    }
}
