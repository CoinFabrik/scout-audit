import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

const FeatureList = [
  {
    title: 'The Tool',
    Svg: require('@site/static/img/scout_tool.svg').default,
    description: (
      <>
        Scout is an extensible, open-source tool designed to help smart contract developers and auditors detect common security issues and deviations from best practices. Currently available for Ink!, Soroban, and Substrate Pallets.
      </>
    ),
  },
  {
    title: 'Security',
    Svg: require('@site/static/img/scout_security.svg').default,
    description: (
      <>
        This tool will help developers write secure and more robust smart contracts. Our interest in this project comes from CoinFabrik's experience in manual auditing and our usage of comparable tools in other blockchains.
      </>
    ),
  },
  {
    title: 'Research',
    Svg: require('@site/static/img/scout_research.svg').default,
    description: (
      <>
        To improve coverage and precision, we persist in research efforts on static and dynamic analysis techniques. Find more about our ongoing research at our associated repository.
      </>
    ),
  },
];

function Feature({Svg, title, description}) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures() {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
