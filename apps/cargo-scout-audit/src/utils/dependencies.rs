use std::{
    collections::{
        HashMap,
        HashSet,
    },
};
use anyhow::{Result, anyhow};
use cargo_metadata::Metadata;

pub struct DependencyGraph{
    packages_ids: Vec<String>,
    packages_numbers_by_package_id: HashMap<String, usize>,
    connection_matrix: Vec<bool>,
}

impl DependencyGraph{
    pub fn new(meta: &Metadata) -> Result<Self>{
        let resolve = meta.resolve
            .clone()
            .ok_or_else(|| anyhow!("Don't have package resolution information to build the dependency graph."))?;
        let packages_ids = meta.packages
            .iter()
            .map(|x| x.id.to_string())
            .collect::<HashSet<_>>()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let packages_numbers_by_package_id = packages_ids
            .iter()
            .enumerate()
            .map(|(x, y)| (y.clone(), x))
            .collect::<HashMap<_, _>>();

        let mut connection_matrix = Vec::<bool>::new();
        let n = packages_ids.len();
        connection_matrix.resize(n * n, false);
        for node in resolve.nodes.iter(){
            let node_id = node.id.to_string();
            let package_number = packages_numbers_by_package_id.get(&node_id)
                .ok_or_else(|| anyhow!("A resolution node references an unknown package ID."))?;
            let deps = node.dependencies
                .iter()
                .map(|x| packages_numbers_by_package_id.get(&x.to_string()));
            for dep in deps{
                if dep.is_none(){
                    Err(anyhow!("Resolution node with ID {} depends on an unknown package ID.", node_id))?;
                }
                let dep = dep.unwrap();
                connection_matrix[package_number * n + dep] = true;
            }
        }

        Ok(Self {
            packages_ids,
            packages_numbers_by_package_id,
            connection_matrix,
        })
    }
    fn get_dependencies<F>(&self, package: usize, cb: F) where F: FnMut(usize){
        let n = self.packages_ids.len();
        if package >= n{
            return;
        }
        let package = package * n;
        (0..n)
            .filter(|i| self.connection_matrix[package + i])
            .for_each(cb);
    }
    fn get_dependents<F>(&self, package: usize, cb: F) where F: FnMut(usize){
        let n = self.packages_ids.len();
        if package >= n{
            return;
        }
        (0..n)
            .filter(|i| self.connection_matrix[package + i * n])
            .for_each(cb);
    }
    pub fn depends_on(&self, package: &str, dependency_to_check: &str, direct: Option<bool>) -> Result<bool>{
        let package = *self.packages_numbers_by_package_id.get(package)
            .ok_or_else(|| anyhow!("Package {} is unknown", package))?;
        let dependency_to_check = *self.packages_numbers_by_package_id.get(dependency_to_check)
            .ok_or_else(|| anyhow!("Package {} is unknown", dependency_to_check))?;

        let n = self.packages_ids.len();
        if !direct.unwrap_or(false){
            let mut visited = Vec::<bool>::new();
            visited.resize(n, false);
            let mut stack = Vec::<usize>::new();
            stack.push(package);
            while !stack.is_empty(){
                let top = *stack.last().unwrap();
                if top == dependency_to_check{
                    return Ok(true);
                }
                stack.pop();
                if visited[top]{
                    continue;
                }
                visited[top] = true;
                self.get_dependencies(top, |dep| {
                    stack.push(dep);
                });
            }
            Ok(false)
        }else{
            let mut ret = false;
            self.get_dependencies(package, |dep| {
                ret = ret || dep == dependency_to_check;
            });
            Ok(ret)
        }
    }
    pub fn list_all_dependants(&self, package: &str) -> Result<Vec<String>>{
        self.list_dependants(package, Some(false))
    }
    pub fn list_dependants(&self, package: &str, direct: Option<bool>) -> Result<Vec<String>>{
        let package = *self.packages_numbers_by_package_id.get(package)
            .ok_or_else(|| anyhow!("Package {} is unknown", package))?;

        let n = self.packages_ids.len();
        let mut visited = Vec::<bool>::new();
        visited.resize(n, false);
        if !direct.unwrap_or(false){
            let mut stack = Vec::<usize>::new();
            stack.push(package);
            while !stack.is_empty(){
                let top = *stack.last().unwrap();
                stack.pop();
                if visited[top]{
                    continue;
                }
                visited[top] = true;
                self.get_dependents(top, |dep| {
                    stack.push(dep);
                });
            }
        }else{
            self.get_dependents(package, |dep| {
                visited[dep] = true;
            });
        }
        
        let ret = visited
            .iter()
            .enumerate()
            .filter(|(_, x)| **x)
            .map(|(i, _)| self.packages_ids[i].clone())
            .collect();

        Ok(ret)
    }
}
