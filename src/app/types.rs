use crate::{
    action::directory_creator::DirectoryCreator,
    components::{
        environment::{
            checks::{home::HomeCheck, structure::StructureCheck, tree::LayoutCheck},
            inference::EnvironmentInference,
        },
        repo::checks::structure::StructureCheck as RepoStructureCheck,
    },
    util::{
        actions::Actions,
        fs::{DirectoryListing, LinkReader, MetadataChecks},
        prompting::Prompter,
    },
    validation::{containment::ContainmentCheck, directory::DirectoryCheck},
};

pub trait App<'a> {
    type MC: MetadataChecks;
    type LR: LinkReader;
    type DL: DirectoryListing;
    type A: Actions;
    type PR: Prompter;
    type EI: EnvironmentInference;

    fn metadata_checks(&self) -> &'a Self::MC;
    fn link_reader(&self) -> &'a Self::LR;
    fn directory_listing(&self) -> &'a Self::DL;
    fn actions(&self) -> &'a Self::A;
    fn prompter(&self) -> &'a Self::PR;
    fn inference(&self) -> &'a Self::EI;

    fn layout_check(
        &self,
        yes: bool,
        create_directories: bool,
    ) -> LayoutCheck<'a, Self::MC, Self::A, Self::PR> {
        let directory_checker = DirectoryCheck::new(self.metadata_checks());
        let directory_creator = DirectoryCreator::new(self.actions(), self.prompter());
        LayoutCheck::new(
            directory_checker,
            directory_creator,
            yes,
            create_directories,
        )
    }

    fn structure_check(&self) -> StructureCheck<'a, Self::MC, Self::LR> {
        StructureCheck::new(ContainmentCheck::new(
            self.metadata_checks(),
            self.link_reader(),
        ))
    }

    fn repo_structure_check(&self) -> RepoStructureCheck<'a, Self::MC> {
        RepoStructureCheck::new(
            DirectoryCheck::new(self.metadata_checks()),
            DirectoryCheck::new(self.metadata_checks()),
        )
    }

    fn home_check(&self) -> HomeCheck<'a, Self::MC> {
        HomeCheck::new(DirectoryCheck::new(self.metadata_checks()))
    }
}
