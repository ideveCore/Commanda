/**
 * Weather conditions mapped by descriptive keys.
 * Each condition contains a corresponding Font Awesome icon and a description in Portuguese.
 * @type {Object.<string, {icon: string, description: string}>}
 */
export const weather_conditions = {
  clear_sky: { icon: "fa-solid fa-sun", description: "Céu limpo" },
  mainly_clear: {
    icon: "fa-solid fa-cloud-sun",
    description: "Predominantemente limpo",
  },
  partly_cloudy: {
    icon: "fa-solid fa-cloud-sun",
    description: "Parcialmente nublado",
  },
  overcast: { icon: "fa-solid fa-cloud", description: "Nublado" },
  fog: { icon: "fa-solid fa-smog", description: "Neblina" },
  depositing_rime_fog: {
    icon: "fa-solid fa-smog",
    description: "Névoa gelada",
  },
  light_drizzle: { icon: "fa-solid fa-cloud-rain", description: "Garoa leve" },
  moderate_drizzle: {
    icon: "fa-solid fa-cloud-rain",
    description: "Garoa moderada",
  },
  dense_drizzle: { icon: "fa-solid fa-cloud-rain", description: "Garoa densa" },
  light_freezing_drizzle: {
    icon: "fa-solid fa-cloud-rain",
    description: "Garoa gelada leve",
  },
  dense_freezing_drizzle: {
    icon: "fa-solid fa-cloud-rain",
    description: "Garoa gelada densa",
  },
  slight_rain: { icon: "fa-solid fa-cloud-rain", description: "Chuva leve" },
  moderate_rain: {
    icon: "fa-solid fa-cloud-rain",
    description: "Chuva moderada",
  },
  heavy_rain: {
    icon: "fa-solid fa-cloud-showers-heavy",
    description: "Chuva forte",
  },
  light_freezing_rain: {
    icon: "fa-solid fa-cloud-rain",
    description: "Chuva leve e gelada",
  },
  heavy_freezing_rain: {
    icon: "fa-solid fa-cloud-showers-heavy",
    description: "Chuva forte e gelada",
  },
  slight_snow_fall: {
    icon: "fa-solid fa-snowflake",
    description: "Queda de neve leve",
  },
  moderate_snow_fall: {
    icon: "fa-solid fa-snowflake",
    description: "Queda de neve moderada",
  },
  heavy_snow_fall: {
    icon: "fa-solid fa-snowflake",
    description: "Queda de neve forte",
  },
  snow_grains: { icon: "fa-solid fa-snowflake", description: "Granizo miúdo" },
  slight_rain_showers: {
    icon: "fa-solid fa-cloud-rain",
    description: "Pancadas de chuva leve",
  },
  moderate_rain_showers: {
    icon: "fa-solid fa-cloud-rain",
    description: "Pancadas de chuva moderadas",
  },
  violent_rain_showers: {
    icon: "fa-solid fa-cloud-showers-heavy",
    description: "Pancadas de chuva violentas",
  },
  slight_snow_showers: {
    icon: "fa-solid fa-snowflake",
    description: "Pancadas de neve leve",
  },
  heavy_snow_showers: {
    icon: "fa-solid fa-snowflake",
    description: "Pancadas de neve forte",
  },
  thunderstorm: { icon: "fa-solid fa-cloud-bolt", description: "Trovoada" },
  thunderstorm_with_slight_hail: {
    icon: "fa-solid fa-cloud-bolt",
    description: "Trovoada com granizo leve",
  },
  thunderstorm_with_heavy_hail: {
    icon: "fa-solid fa-cloud-bolt",
    description: "Trovoada com granizo forte",
  },
};

/**
 * Formats a number to the Brazilian standard with two decimal places.
 *
 * @param {number} number - The number to be formatted.
 * @returns {string} The number formatted as a string in the pt-BR format.
 */
export const format_number = (number) => {
  return number.toLocaleString("pt-BR", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
};
